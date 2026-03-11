// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::models::SidecarMode;

#[derive(Debug, Clone)]
pub struct BudgetPolicy {
    pub mode: SidecarMode,
    pub max_local_latency_ms: u64,
    pub max_cloud_latency_ms: u64,
    pub max_requests_per_day: u64,
    pub max_tokens_per_request: u64,
    pub max_monthly_budget_usd: f64,
}

impl Default for BudgetPolicy {
    fn default() -> Self {
        Self {
            mode: SidecarMode::LocalOnly,
            max_local_latency_ms: 250,
            max_cloud_latency_ms: 1_500,
            max_requests_per_day: 1_000,
            max_tokens_per_request: 8_192,
            max_monthly_budget_usd: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BudgetState {
    pub requests_today: u64,
    pub estimated_monthly_spend_usd: f64,
}

impl BudgetPolicy {
    pub fn allow_request(
        &self,
        state: &mut BudgetState,
        is_cloud: bool,
        token_estimate: u64,
    ) -> Result<(), String> {
        if state.requests_today >= self.max_requests_per_day {
            return Err("daily request budget exhausted".to_string());
        }
        if token_estimate > self.max_tokens_per_request {
            return Err("token budget exceeded for request".to_string());
        }
        if is_cloud && self.mode == SidecarMode::LocalOnly {
            return Err("cloud requests are disabled by local-only policy".to_string());
        }
        if is_cloud && state.estimated_monthly_spend_usd >= self.max_monthly_budget_usd {
            return Err("monthly cloud budget exhausted".to_string());
        }
        state.requests_today += 1;
        Ok(())
    }

    pub fn within_latency_budget(&self, elapsed_ms: u64, is_cloud: bool) -> bool {
        if is_cloud {
            elapsed_ms <= self.max_cloud_latency_ms
        } else {
            elapsed_ms <= self.max_local_latency_ms
        }
    }
}
