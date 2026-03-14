// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! AI-powered analysis using Anthropic Claude.
//! Performs semantic intent drift detection, contract suggestion,
//! example generation, and intent drafting.

use serde::Deserialize;
use vibe_indexer::FunctionMeta;

use crate::config::SidecarConfig;
use crate::llm_cache::ResponseCache;
use crate::llm_client::AnthropicClient;
use crate::models::{CandidateSuggestion, EvidenceRef, FindingSeverity, IntentFinding};
use crate::prompts;

const MAX_TOKENS_DRIFT: u32 = 512;
const MAX_TOKENS_SUGGEST: u32 = 1024;
const DRIFT_CONFIDENCE_THRESHOLD: f32 = 0.6;

#[derive(Deserialize)]
struct DriftResponse {
    aligned: bool,
    confidence: f64,
    rationale: String,
    drift_description: String,
}

#[derive(Deserialize)]
struct ContractSuggestionItem {
    #[serde(rename = "type")]
    contract_type: String,
    expression: String,
    rationale: String,
    confidence: f64,
}

#[derive(Deserialize)]
struct ContractSuggestionsResponse {
    suggestions: Vec<ContractSuggestionItem>,
}

#[derive(Deserialize)]
struct ExampleItem {
    input: String,
    expected: String,
    rationale: String,
    confidence: f64,
}

#[derive(Deserialize)]
struct ExamplesResponse {
    examples: Vec<ExampleItem>,
}

#[derive(Deserialize)]
struct IntentSuggestionResponse {
    intent_text: String,
    confidence: f64,
    rationale: String,
}

pub struct AiAnalyzer {
    client: AnthropicClient,
    cache: ResponseCache,
    config: SidecarConfig,
}

impl AiAnalyzer {
    pub fn new(config: &SidecarConfig, index_root: &std::path::Path) -> Option<Self> {
        let api_key = config.api_key.as_ref()?;
        if api_key.is_empty() {
            return None;
        }

        let client = AnthropicClient::new(
            api_key.clone(),
            config.model.clone(),
            config.endpoint.clone(),
            30_000, // per-request timeout (30s for Claude API calls)
        );
        let cache = ResponseCache::new(index_root, config.cache_ttl_hours);

        Some(Self {
            client,
            cache,
            config: config.clone(),
        })
    }

    pub async fn analyze_intent_drift(
        &self,
        meta: &FunctionMeta,
        source_code: &str,
    ) -> Vec<IntentFinding> {
        let intent_text = match &meta.intent_text {
            Some(t) => t,
            None => return Vec::new(),
        };

        if let Some(cached) = self.cache.get(
            &meta.signature_hash,
            intent_text,
            &self.config.model,
            prompts::PROMPT_VERSION,
        ) {
            if let Some(finding) = self.parse_drift_response(&cached, meta) {
                return finding;
            }
        }

        let code = self.maybe_redact(source_code);
        let contracts = self.format_contracts(meta);
        let prompt = prompts::drift_detection_prompt(
            &meta.function_name,
            intent_text,
            &code,
            &meta.effects_declared,
            &contracts,
        );

        match self
            .client
            .send_message(prompts::SYSTEM_PROMPT, &prompt, MAX_TOKENS_DRIFT)
            .await
        {
            Ok(resp) => {
                self.cache.put(
                    &meta.signature_hash,
                    intent_text,
                    &self.config.model,
                    prompts::PROMPT_VERSION,
                    &resp.text,
                );
                self.parse_drift_response(&resp.text, meta)
                    .unwrap_or_default()
            }
            Err(_) => Vec::new(),
        }
    }

    pub async fn suggest_contracts(
        &self,
        meta: &FunctionMeta,
        source_code: &str,
    ) -> Vec<CandidateSuggestion> {
        let code = self.maybe_redact(source_code);
        let contracts = self.format_contracts(meta);
        let prompt =
            prompts::contract_suggestion_prompt(&meta.function_name, &code, &contracts);

        match self
            .client
            .send_message(prompts::SYSTEM_PROMPT, &prompt, MAX_TOKENS_SUGGEST)
            .await
        {
            Ok(resp) => self.parse_contract_suggestions(&resp.text, meta),
            Err(_) => Vec::new(),
        }
    }

    pub async fn suggest_examples(
        &self,
        meta: &FunctionMeta,
        source_code: &str,
    ) -> Vec<CandidateSuggestion> {
        let code = self.maybe_redact(source_code);
        let prompt = prompts::example_suggestion_prompt(&meta.function_name, &code);

        match self
            .client
            .send_message(prompts::SYSTEM_PROMPT, &prompt, MAX_TOKENS_SUGGEST)
            .await
        {
            Ok(resp) => self.parse_example_suggestions(&resp.text, meta),
            Err(_) => Vec::new(),
        }
    }

    pub async fn suggest_intent(
        &self,
        meta: &FunctionMeta,
        source_code: &str,
    ) -> Vec<CandidateSuggestion> {
        let code = self.maybe_redact(source_code);
        let prompt = prompts::intent_suggestion_prompt(&meta.function_name, &code);

        match self
            .client
            .send_message(prompts::SYSTEM_PROMPT, &prompt, MAX_TOKENS_SUGGEST)
            .await
        {
            Ok(resp) => self.parse_intent_suggestion(&resp.text, meta),
            Err(_) => Vec::new(),
        }
    }

    fn parse_drift_response(
        &self,
        text: &str,
        meta: &FunctionMeta,
    ) -> Option<Vec<IntentFinding>> {
        let json_text = extract_json(text)?;
        let resp: DriftResponse = serde_json::from_str(json_text).ok()?;

        if resp.aligned && resp.confidence as f32 >= DRIFT_CONFIDENCE_THRESHOLD {
            return Some(Vec::new());
        }

        if !resp.aligned {
            Some(vec![IntentFinding {
                code: "W0801".to_string(),
                severity: FindingSeverity::Warning,
                message: format!(
                    "possible intent drift in `{}`: {}",
                    meta.function_name, resp.drift_description
                ),
                confidence: resp.confidence as f32,
                evidence: vec![EvidenceRef {
                    file: meta.file.clone(),
                    symbol: Some(meta.function_name.clone()),
                    detail: resp.rationale,
                }],
                incomplete: false,
            }])
        } else {
            Some(Vec::new())
        }
    }

    fn parse_contract_suggestions(
        &self,
        text: &str,
        meta: &FunctionMeta,
    ) -> Vec<CandidateSuggestion> {
        let json_text = match extract_json(text) {
            Some(t) => t,
            None => return Vec::new(),
        };
        let resp: ContractSuggestionsResponse = match serde_json::from_str(json_text) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        resp.suggestions
            .into_iter()
            .map(|s| CandidateSuggestion {
                id: format!(
                    "contract:{}:{}:{}",
                    meta.file, meta.function_name, s.contract_type
                ),
                title: format!(
                    "Add @{} for `{}`",
                    s.contract_type, meta.function_name
                ),
                summary: format!("@{} {}\n{}", s.contract_type, s.expression, s.rationale),
                confidence: s.confidence as f32,
                evidence: vec![EvidenceRef {
                    file: meta.file.clone(),
                    symbol: Some(meta.function_name.clone()),
                    detail: s.rationale,
                }],
                verified: false,
            })
            .collect()
    }

    fn parse_example_suggestions(
        &self,
        text: &str,
        meta: &FunctionMeta,
    ) -> Vec<CandidateSuggestion> {
        let json_text = match extract_json(text) {
            Some(t) => t,
            None => return Vec::new(),
        };
        let resp: ExamplesResponse = match serde_json::from_str(json_text) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        resp.examples
            .into_iter()
            .map(|e| CandidateSuggestion {
                id: format!("example:{}:{}:{}", meta.file, meta.function_name, e.input),
                title: format!("Add @examples case: {} => {}", e.input, e.expected),
                summary: format!("{} => {}\n{}", e.input, e.expected, e.rationale),
                confidence: e.confidence as f32,
                evidence: vec![EvidenceRef {
                    file: meta.file.clone(),
                    symbol: Some(meta.function_name.clone()),
                    detail: e.rationale,
                }],
                verified: false,
            })
            .collect()
    }

    fn parse_intent_suggestion(
        &self,
        text: &str,
        meta: &FunctionMeta,
    ) -> Vec<CandidateSuggestion> {
        let json_text = match extract_json(text) {
            Some(t) => t,
            None => return Vec::new(),
        };
        let resp: IntentSuggestionResponse = match serde_json::from_str(json_text) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        vec![CandidateSuggestion {
            id: format!("intent:{}:{}", meta.file, meta.function_name),
            title: format!("Add @intent for `{}`", meta.function_name),
            summary: format!(
                "@intent \"{}\"\n{}",
                resp.intent_text, resp.rationale
            ),
            confidence: resp.confidence as f32,
            evidence: vec![EvidenceRef {
                file: meta.file.clone(),
                symbol: Some(meta.function_name.clone()),
                detail: resp.rationale,
            }],
            verified: false,
        }]
    }

    fn maybe_redact(&self, source: &str) -> String {
        if self.config.redact_strings {
            crate::llm_client::redact_string_literals(source)
        } else {
            source.to_string()
        }
    }

    fn format_contracts(&self, meta: &FunctionMeta) -> String {
        let mut parts = Vec::new();
        for eff in &meta.effects_declared {
            parts.push(format!("@effect {eff}"));
        }
        if let Some(ref intent) = meta.intent_text {
            parts.push(format!("@intent \"{intent}\""));
        }
        if parts.is_empty() {
            "none".to_string()
        } else {
            parts.join("\n")
        }
    }
}

/// Extract the first JSON object from a response that may contain markdown fences.
fn extract_json(text: &str) -> Option<&str> {
    let trimmed = text.trim();

    if trimmed.starts_with('{') {
        let end = find_matching_brace(trimmed)?;
        return Some(&trimmed[..=end]);
    }

    if let Some(start) = trimmed.find('{') {
        let rest = &trimmed[start..];
        let end = find_matching_brace(rest)?;
        return Some(&rest[..=end]);
    }

    None
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;

    for (i, ch) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}
