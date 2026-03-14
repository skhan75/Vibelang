// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! BYOK (Bring Your Own Key) configuration for the AI sidecar.
//!
//! Key resolution precedence:
//! 1. `ANTHROPIC_API_KEY` environment variable
//! 2. `~/.config/vibe/sidecar.toml` (global per-machine)
//! 3. Project `vibe.toml` `[sidecar]` section

use std::path::Path;
#[cfg(feature = "cloud")]
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const ENV_API_KEY: &str = "ANTHROPIC_API_KEY";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const DEFAULT_ENDPOINT: &str = "https://api.anthropic.com";
const DEFAULT_CACHE_TTL_HOURS: u64 = 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub endpoint: String,
    pub cache_ttl_hours: u64,
    pub redact_strings: bool,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: DEFAULT_MODEL.to_string(),
            endpoint: DEFAULT_ENDPOINT.to_string(),
            cache_ttl_hours: DEFAULT_CACHE_TTL_HOURS,
            redact_strings: true,
        }
    }
}

#[cfg(feature = "cloud")]
#[derive(Debug, Deserialize, Default)]
struct TomlSidecarSection {
    api_key: Option<String>,
    model: Option<String>,
    endpoint: Option<String>,
    cache_ttl_hours: Option<u64>,
    redact_strings: Option<bool>,
}

#[cfg(feature = "cloud")]
#[derive(Debug, Deserialize, Default)]
struct TomlWrapper {
    sidecar: Option<TomlSidecarSection>,
}

impl SidecarConfig {
    /// Resolve configuration using BYOK precedence:
    /// 1. `ANTHROPIC_API_KEY` env var
    /// 2. `~/.config/vibe/sidecar.toml`
    /// 3. Project `vibe.toml` `[sidecar]` section
    pub fn resolve(#[allow(unused_variables)] project_root: Option<&Path>) -> Self {
        let mut config = Self::default();

        #[cfg(feature = "cloud")]
        {
            if let Some(global) = Self::load_global_config() {
                config.merge_toml_section(&global);
            }

            if let Some(root) = project_root {
                let project_toml = root.join("vibe.toml");
                if let Some(section) = Self::load_toml_section(&project_toml) {
                    if section.api_key.is_some() {
                        eprintln!(
                            "warning: api_key found in project vibe.toml — \
                             consider using ANTHROPIC_API_KEY env var or \
                             ~/.config/vibe/sidecar.toml to avoid committing secrets"
                        );
                    }
                    config.merge_toml_section(&section);
                }
            }
        }

        if let Ok(key) = std::env::var(ENV_API_KEY) {
            if !key.is_empty() {
                config.api_key = Some(key);
            }
        }

        config
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key.as_ref().is_some_and(|k| !k.is_empty())
    }

    #[cfg(feature = "cloud")]
    fn global_config_path() -> Option<PathBuf> {
        dirs_path().map(|d| d.join("sidecar.toml"))
    }

    #[cfg(feature = "cloud")]
    fn load_global_config() -> Option<TomlSidecarSection> {
        let path = Self::global_config_path()?;
        Self::load_toml_section(&path)
    }

    #[cfg(feature = "cloud")]
    fn load_toml_section(path: &Path) -> Option<TomlSidecarSection> {
        let text = std::fs::read_to_string(path).ok()?;
        let wrapper: TomlWrapper = toml::from_str(&text).ok()?;
        wrapper.sidecar.or_else(|| {
            // Allow top-level keys in global sidecar.toml
            toml::from_str::<TomlSidecarSection>(&text).ok()
        })
    }

    #[cfg(feature = "cloud")]
    fn merge_toml_section(&mut self, section: &TomlSidecarSection) {
        if let Some(ref key) = section.api_key {
            self.api_key = Some(key.clone());
        }
        if let Some(ref model) = section.model {
            self.model = model.clone();
        }
        if let Some(ref endpoint) = section.endpoint {
            self.endpoint = endpoint.clone();
        }
        if let Some(ttl) = section.cache_ttl_hours {
            self.cache_ttl_hours = ttl;
        }
        if let Some(redact) = section.redact_strings {
            self.redact_strings = redact;
        }
    }
}

#[cfg(feature = "cloud")]
fn dirs_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".config").join("vibe"))
}
