// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SidecarTelemetry {
    pub requests: u64,
    pub findings_emitted: u64,
    pub partial_responses: u64,
    pub last_elapsed_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct TelemetrySink {
    enabled: bool,
    data: SidecarTelemetry,
}

impl TelemetrySink {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            data: SidecarTelemetry::default(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn record_request(&mut self, elapsed_ms: u64, findings_count: usize, incomplete: bool) {
        if !self.enabled {
            return;
        }
        self.data.requests += 1;
        self.data.findings_emitted += findings_count as u64;
        self.data.last_elapsed_ms = elapsed_ms;
        if incomplete {
            self.data.partial_responses += 1;
        }
    }

    pub fn snapshot(&self) -> SidecarTelemetry {
        self.data.clone()
    }

    pub fn write_json(&self, path: &Path) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        let json = serde_json::to_string_pretty(&self.data)
            .map_err(|e| format!("failed to serialize telemetry: {e}"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create telemetry directory: {e}"))?;
        }
        fs::write(path, json).map_err(|e| format!("failed to write telemetry: {e}"))
    }
}
