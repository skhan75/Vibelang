// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Disk-based response cache for AI sidecar LLM calls.
//! Key = SHA-256(signature_hash + intent_text + model + prompt_version).
//! TTL-based invalidation (default 24h).

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    response_text: String,
    created_at_epoch_secs: u64,
    model: String,
    prompt_version: u32,
}

#[derive(Debug, Clone)]
pub struct ResponseCache {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl ResponseCache {
    pub fn new(index_root: &Path, ttl_hours: u64) -> Self {
        let cache_dir = index_root.join("cache").join("sidecar");
        Self {
            cache_dir,
            ttl: Duration::from_secs(ttl_hours * 3600),
        }
    }

    pub fn get(
        &self,
        signature_hash: &str,
        intent_text: &str,
        model: &str,
        prompt_version: u32,
    ) -> Option<String> {
        let key = self.compute_key(signature_hash, intent_text, model, prompt_version);
        let path = self.cache_dir.join(format!("{key}.json"));
        let text = std::fs::read_to_string(&path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&text).ok()?;

        if entry.prompt_version != prompt_version || entry.model != model {
            return None;
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now.saturating_sub(entry.created_at_epoch_secs) > self.ttl.as_secs() {
            let _ = std::fs::remove_file(&path);
            return None;
        }

        Some(entry.response_text)
    }

    pub fn put(
        &self,
        signature_hash: &str,
        intent_text: &str,
        model: &str,
        prompt_version: u32,
        response_text: &str,
    ) {
        let key = self.compute_key(signature_hash, intent_text, model, prompt_version);
        let path = self.cache_dir.join(format!("{key}.json"));

        if std::fs::create_dir_all(&self.cache_dir).is_err() {
            return;
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = CacheEntry {
            response_text: response_text.to_string(),
            created_at_epoch_secs: now,
            model: model.to_string(),
            prompt_version,
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            let _ = std::fs::write(&path, json);
        }
    }

    fn compute_key(
        &self,
        signature_hash: &str,
        intent_text: &str,
        model: &str,
        prompt_version: u32,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(signature_hash.as_bytes());
        hasher.update(b"|");
        hasher.update(intent_text.as_bytes());
        hasher.update(b"|");
        hasher.update(model.as_bytes());
        hasher.update(b"|");
        hasher.update(prompt_version.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }
}
