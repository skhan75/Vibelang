// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod config;
pub mod index_access;
#[cfg(feature = "cloud")]
pub mod ai_analyzer;
#[cfg(feature = "cloud")]
pub mod llm_cache;
#[cfg(feature = "cloud")]
pub mod llm_client;
pub mod models;
pub mod policy;
#[cfg(feature = "cloud")]
pub mod prompts;
pub mod service;
pub mod telemetry;

pub use config::SidecarConfig;
pub use models::{
    CandidateSuggestion, EvidenceRef, FindingSeverity, IntentFinding, IntentLintRequest,
    IntentLintResponse, SidecarMode,
};
pub use policy::{BudgetPolicy, BudgetState};
pub use service::SidecarService;
pub use telemetry::{SidecarTelemetry, TelemetrySink};
