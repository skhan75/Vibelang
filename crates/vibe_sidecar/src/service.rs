use std::collections::BTreeSet;
use std::path::Path;
use std::time::Instant;

use crate::index_access::ReadOnlyIndex;
use crate::models::{
    CandidateSuggestion, EvidenceRef, FindingSeverity, IntentFinding, IntentLintRequest,
    IntentLintResponse, SidecarMode,
};
use crate::policy::{BudgetPolicy, BudgetState};
use crate::telemetry::TelemetrySink;

#[derive(Debug, Clone)]
pub struct SidecarService {
    index: ReadOnlyIndex,
    policy: BudgetPolicy,
    budget_state: BudgetState,
    telemetry: TelemetrySink,
}

impl SidecarService {
    pub fn new(index_root: &Path, policy: BudgetPolicy, telemetry_enabled: bool) -> Result<Self, String> {
        Ok(Self {
            index: ReadOnlyIndex::open(index_root)?,
            policy,
            budget_state: BudgetState::default(),
            telemetry: TelemetrySink::new(telemetry_enabled),
        })
    }

    pub fn mode(&self) -> SidecarMode {
        self.policy.mode
    }

    pub fn telemetry(&self) -> &TelemetrySink {
        &self.telemetry
    }

    pub fn telemetry_mut(&mut self) -> &mut TelemetrySink {
        &mut self.telemetry
    }

    pub fn lint_intent(&mut self, request: &IntentLintRequest) -> IntentLintResponse {
        let start = Instant::now();
        let mut incomplete = false;
        let mut findings = Vec::new();
        let mut suggestions = Vec::new();

        if let Err(err) = self.policy.allow_request(&mut self.budget_state, false, 0) {
            findings.push(IntentFinding {
                code: "I9001".to_string(),
                severity: FindingSeverity::Warning,
                message: format!("intent lint budget guard: {err}"),
                confidence: 1.0,
                evidence: Vec::new(),
                incomplete: true,
            });
            let elapsed_ms = start.elapsed().as_millis() as u64;
            self.telemetry.record_request(elapsed_ms, findings.len(), true);
            return IntentLintResponse {
                findings,
                suggestions,
                mode: self.policy.mode,
                incomplete: true,
                elapsed_ms,
            };
        }

        let changed_file_set = request
            .changed_files
            .iter()
            .map(|f| f.as_str())
            .collect::<BTreeSet<_>>();

        for meta in self.index.all_functions() {
            if request.changed_only
                && !changed_file_set.contains(meta.file.as_str())
            {
                continue;
            }

            if meta.is_public && meta.intent_text.is_none() {
                findings.push(IntentFinding {
                    code: "I5001".to_string(),
                    severity: FindingSeverity::Warning,
                    message: format!(
                        "public function `{}` is missing `@intent`",
                        meta.function_name
                    ),
                    confidence: 0.96,
                    evidence: vec![EvidenceRef {
                        file: meta.file.clone(),
                        symbol: Some(meta.function_name.clone()),
                        detail: "public api has no intent metadata".to_string(),
                    }],
                    incomplete: false,
                });
                if request.include_suggestions {
                    suggestions.push(CandidateSuggestion {
                        id: format!("intent:{}:{}", meta.file, meta.function_name),
                        title: format!("Add @intent for `{}`", meta.function_name),
                        summary: "Add a single-sentence behavior intent before executable statements."
                            .to_string(),
                        confidence: 0.78,
                        evidence: vec![EvidenceRef {
                            file: meta.file.clone(),
                            symbol: Some(meta.function_name.clone()),
                            detail: "missing intent on public function".to_string(),
                        }],
                        verified: false,
                    });
                }
            }

            if let Some(intent) = &meta.intent_text {
                if looks_vague(intent) {
                    findings.push(IntentFinding {
                        code: "I5002".to_string(),
                        severity: FindingSeverity::Warning,
                        message: format!(
                            "intent text for `{}` is likely too vague",
                            meta.function_name
                        ),
                        confidence: 0.71,
                        evidence: vec![EvidenceRef {
                            file: meta.file.clone(),
                            symbol: Some(meta.function_name.clone()),
                            detail: format!("intent text: `{intent}`"),
                        }],
                        incomplete: false,
                    });
                }
            }
        }

        for mismatch in self.index.effect_mismatch_findings() {
            if request.changed_only && !changed_file_set.contains(mismatch.file.as_str()) {
                continue;
            }
            findings.push(IntentFinding {
                code: "I5003".to_string(),
                severity: FindingSeverity::Info,
                message: format!(
                    "effect drift for `{}`: declared_only={:?}, observed_only={:?}",
                    mismatch.function_name, mismatch.declared_only, mismatch.observed_only
                ),
                confidence: 0.84,
                evidence: vec![EvidenceRef {
                    file: mismatch.file.clone(),
                    symbol: Some(mismatch.function_name.clone()),
                    detail: "effect mismatch from semantic index".to_string(),
                }],
                incomplete: false,
            });
        }

        for missing in self.index.public_functions_missing_examples() {
            if request.changed_only && !changed_file_set.contains(missing.file.as_str()) {
                continue;
            }
            findings.push(IntentFinding {
                code: "I5004".to_string(),
                severity: FindingSeverity::Info,
                message: format!(
                    "public function `{}` has no `@examples` coverage",
                    missing.function_name
                ),
                confidence: 0.8,
                evidence: vec![EvidenceRef {
                    file: missing.file.clone(),
                    symbol: Some(missing.function_name.clone()),
                    detail: "missing executable examples".to_string(),
                }],
                incomplete: false,
            });
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;
        if !self.policy.within_latency_budget(elapsed_ms, false) {
            incomplete = true;
            findings.push(IntentFinding {
                code: "I9002".to_string(),
                severity: FindingSeverity::Warning,
                message: "intent lint exceeded local latency budget; results may be partial".to_string(),
                confidence: 1.0,
                evidence: Vec::new(),
                incomplete: true,
            });
        }
        self.telemetry
            .record_request(elapsed_ms, findings.len(), incomplete);

        IntentLintResponse {
            findings,
            suggestions,
            mode: self.policy.mode,
            incomplete,
            elapsed_ms,
        }
    }
}

fn looks_vague(intent: &str) -> bool {
    let text = intent.trim().to_ascii_lowercase();
    if text.is_empty() {
        return true;
    }
    let words = text.split_whitespace().count();
    words < 3
        || text == "does stuff"
        || text.contains("todo")
        || text.contains("something")
        || text.contains("whatever")
}
