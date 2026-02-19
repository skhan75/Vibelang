use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidecarMode {
    LocalOnly,
    Hybrid,
    Cloud,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub file: String,
    pub symbol: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentFinding {
    pub code: String,
    pub severity: FindingSeverity,
    pub message: String,
    pub confidence: f32,
    pub evidence: Vec<EvidenceRef>,
    pub incomplete: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateSuggestion {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub confidence: f32,
    pub evidence: Vec<EvidenceRef>,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IntentLintRequest {
    pub query: Option<String>,
    pub changed_only: bool,
    pub changed_files: Vec<String>,
    pub include_suggestions: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentLintResponse {
    pub findings: Vec<IntentFinding>,
    pub suggestions: Vec<CandidateSuggestion>,
    pub mode: SidecarMode,
    pub incomplete: bool,
    pub elapsed_ms: u64,
}
