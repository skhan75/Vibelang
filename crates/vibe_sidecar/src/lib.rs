// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod index_access;
pub mod models;
pub mod policy;
pub mod service;
pub mod telemetry;

pub use models::{
    CandidateSuggestion, EvidenceRef, FindingSeverity, IntentFinding, IntentLintRequest,
    IntentLintResponse, SidecarMode,
};
pub use policy::{BudgetPolicy, BudgetState};
pub use service::SidecarService;
pub use telemetry::{SidecarTelemetry, TelemetrySink};
