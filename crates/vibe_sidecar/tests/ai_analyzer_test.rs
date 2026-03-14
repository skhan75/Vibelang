// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for AI analyzer response parsing and cache behavior.
//! These tests use mock LLM responses (no network calls).

#[cfg(feature = "cloud")]
mod cloud_tests {
    use std::path::PathBuf;
    use std::sync::Mutex;
    use vibe_sidecar::config::SidecarConfig;
    use vibe_sidecar::llm_cache::ResponseCache;
    use vibe_sidecar::llm_client::redact_string_literals;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn config_resolves_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set("ANTHROPIC_API_KEY", "sk-test-key-123");
        let config = SidecarConfig::resolve(None);
        assert!(config.has_api_key());
        assert_eq!(config.api_key.as_deref(), Some("sk-test-key-123"));
    }

    #[test]
    fn config_no_key_returns_none() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::remove("ANTHROPIC_API_KEY");
        let config = SidecarConfig::resolve(None);
        assert!(!config.has_api_key());
    }

    #[test]
    fn config_defaults_are_correct() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::remove("ANTHROPIC_API_KEY");
        let config = SidecarConfig::resolve(None);
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.endpoint, "https://api.anthropic.com");
        assert_eq!(config.cache_ttl_hours, 24);
        assert!(config.redact_strings);
    }

    #[test]
    fn cache_put_and_get() {
        let dir = tempdir();
        let cache = ResponseCache::new(dir.path(), 24);

        cache.put("sig123", "sort ascending", "claude-sonnet-4-20250514", 1, r#"{"aligned":true}"#);
        let result = cache.get("sig123", "sort ascending", "claude-sonnet-4-20250514", 1);
        assert_eq!(result, Some(r#"{"aligned":true}"#.to_string()));
    }

    #[test]
    fn cache_miss_on_different_model() {
        let dir = tempdir();
        let cache = ResponseCache::new(dir.path(), 24);

        cache.put("sig123", "sort ascending", "claude-sonnet-4-20250514", 1, r#"{"aligned":true}"#);
        let result = cache.get("sig123", "sort ascending", "claude-3-opus-20240229", 1);
        assert_eq!(result, None);
    }

    #[test]
    fn cache_miss_on_different_prompt_version() {
        let dir = tempdir();
        let cache = ResponseCache::new(dir.path(), 24);

        cache.put("sig123", "sort ascending", "claude-sonnet-4-20250514", 1, r#"{"aligned":true}"#);
        let result = cache.get("sig123", "sort ascending", "claude-sonnet-4-20250514", 2);
        assert_eq!(result, None);
    }

    #[test]
    fn cache_expired_entry_returns_none() {
        let dir = tempdir();
        let cache = ResponseCache::new(dir.path(), 24);

        cache.put("sig123", "sort ascending", "claude-sonnet-4-20250514", 1, r#"{"aligned":true}"#);

        // Manually tamper with the cache file to set created_at far in the past
        let cache_dir = dir.path().join("cache").join("sidecar");
        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                let text = std::fs::read_to_string(entry.path()).unwrap();
                let tampered = text.replace(
                    &{
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        // Replace any recent timestamp with one 48h in the past
                        format!("\"created_at_epoch_secs\":{now}")
                    },
                    &format!("\"created_at_epoch_secs\":{}", 1000),
                );
                std::fs::write(entry.path(), tampered).unwrap();
            }
        }

        let result = cache.get("sig123", "sort ascending", "claude-sonnet-4-20250514", 1);
        assert_eq!(result, None);
    }

    #[test]
    fn redact_string_literals_strips_content() {
        let input = r#"let msg = "hello world"
let x = 42"#;
        let output = redact_string_literals(input);
        assert!(output.contains(r#""...""#));
        assert!(output.contains("42"));
        assert!(!output.contains("hello world"));
    }

    #[test]
    fn redact_preserves_code_structure() {
        let input = r#"fn greet(name: String) -> String {
  @intent "greet the user by name"
  format("Hello, {}", name)
}"#;
        let output = redact_string_literals(input);
        assert!(output.contains("fn greet"));
        assert!(output.contains("format"));
        assert!(!output.contains("greet the user by name"));
        assert!(!output.contains("Hello, {}"));
    }

    #[test]
    fn redact_handles_empty_strings() {
        let input = r#"let s = """#;
        let output = redact_string_literals(input);
        assert_eq!(output, r#"let s = "...""#);
    }

    // --- Helpers ---

    struct EnvGuard {
        key: String,
        prev: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &str, val: &str) -> Self {
            let prev = std::env::var(key).ok();
            std::env::set_var(key, val);
            Self {
                key: key.to_string(),
                prev,
            }
        }

        fn remove(key: &str) -> Self {
            let prev = std::env::var(key).ok();
            std::env::remove_var(key);
            Self {
                key: key.to_string(),
                prev,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(val) => std::env::set_var(&self.key, val),
                None => std::env::remove_var(&self.key),
            }
        }
    }

    fn tempdir() -> TempDir {
        TempDir::new()
    }

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!("vibe_sidecar_test_{}", std::process::id()));
            path.push(format!("{}", rand_u64()));
            let _ = std::fs::create_dir_all(&path);
            Self(path)
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn rand_u64() -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;
        let mut h = DefaultHasher::new();
        SystemTime::now().hash(&mut h);
        std::thread::current().id().hash(&mut h);
        h.finish()
    }
}
