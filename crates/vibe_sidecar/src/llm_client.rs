// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Async HTTP client for the Anthropic Messages API.
//! All traffic goes directly from the developer's machine to Anthropic.
//! VibeLang has no centralized proxy.

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

const ANTHROPIC_VERSION: &str = "2023-06-01";
const MAX_RETRIES: u32 = 1;

#[derive(Debug)]
pub enum LlmError {
    NoApiKey,
    Http(String),
    Timeout,
    RateLimited,
    BadResponse(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoApiKey => write!(f, "no API key configured"),
            Self::Http(msg) => write!(f, "HTTP error: {msg}"),
            Self::Timeout => write!(f, "request timed out"),
            Self::RateLimited => write!(f, "rate limited by API"),
            Self::BadResponse(msg) => write!(f, "bad API response: {msg}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnthropicClient {
    api_key: String,
    model: String,
    endpoint: String,
    timeout: Duration,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[derive(Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

pub struct LlmResponse {
    pub text: String,
    pub usage: Option<Usage>,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: String, endpoint: String, timeout_ms: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .unwrap_or_default();

        Self {
            api_key,
            model,
            endpoint,
            timeout: Duration::from_millis(timeout_ms),
            client,
        }
    }

    pub async fn send_message(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: u32,
    ) -> Result<LlmResponse, LlmError> {
        if self.api_key.is_empty() {
            return Err(LlmError::NoApiKey);
        }

        let url = format!("{}/v1/messages", self.endpoint.trim_end_matches('/'));
        let body = MessagesRequest {
            model: &self.model,
            max_tokens,
            system: system_prompt,
            messages: vec![Message {
                role: "user",
                content: user_prompt,
            }],
        };

        let mut last_err = LlmError::Http("no attempt made".to_string());

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let backoff = Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }

            let result = self
                .client
                .post(&url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("content-type", "application/json")
                .timeout(self.timeout)
                .json(&body)
                .send()
                .await;

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        let api_resp: MessagesResponse = resp
                            .json()
                            .await
                            .map_err(|e| LlmError::BadResponse(e.to_string()))?;

                        let text = api_resp
                            .content
                            .into_iter()
                            .filter_map(|b| b.text)
                            .collect::<Vec<_>>()
                            .join("");

                        if text.is_empty() {
                            return Err(LlmError::BadResponse("empty response".to_string()));
                        }

                        return Ok(LlmResponse {
                            text,
                            usage: api_resp.usage,
                        });
                    }

                    if status.as_u16() == 429 {
                        last_err = LlmError::RateLimited;
                        continue;
                    }
                    if status.is_server_error() {
                        let body_text = resp.text().await.unwrap_or_default();
                        last_err = LlmError::Http(format!("{status}: {body_text}"));
                        continue;
                    }

                    let body_text = resp.text().await.unwrap_or_default();
                    return Err(LlmError::Http(format!("{status}: {body_text}")));
                }
                Err(e) if e.is_timeout() => {
                    last_err = LlmError::Timeout;
                    continue;
                }
                Err(e) => {
                    return Err(LlmError::Http(e.to_string()));
                }
            }
        }

        Err(last_err)
    }
}

/// Strip string literal contents for privacy before sending to the API.
/// Replaces the contents of double-quoted strings with `"..."`.
pub fn redact_string_literals(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            result.push('"');
            result.push_str("...");
            let mut escaped = false;
            for inner in chars.by_ref() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if inner == '\\' {
                    escaped = true;
                    continue;
                }
                if inner == '"' {
                    result.push('"');
                    break;
                }
            }
        } else if ch == '/' && chars.peek() == Some(&'/') {
            // Strip single-line comments
            result.push_str("// ...");
            for inner in chars.by_ref() {
                if inner == '\n' {
                    result.push('\n');
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_strips_string_contents() {
        let input = r#"println("hello world")"#;
        let output = redact_string_literals(input);
        assert_eq!(output, r#"println("...")"#);
    }

    #[test]
    fn redact_strips_comments() {
        let input = "x + 1 // add one\ny + 2";
        let output = redact_string_literals(input);
        assert_eq!(output, "x + 1 // ...\ny + 2");
    }

    #[test]
    fn redact_handles_escaped_quotes() {
        let input = r#"msg("say \"hi\"")"#;
        let output = redact_string_literals(input);
        assert_eq!(output, r#"msg("...")"#);
    }
}
