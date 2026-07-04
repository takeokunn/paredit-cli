use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefactorOperation {
    Rename,
    Remove,
    Move,
    Signature,
}

impl RefactorOperation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rename => "rename",
            Self::Remove => "remove",
            Self::Move => "move",
            Self::Signature => "signature",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationPhase {
    Pre,
    Post,
}

impl VerificationPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pre => "pre",
            Self::Post => "post",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefactorRiskLevel {
    Info,
    Warning,
    Error,
}

impl RefactorRiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RefactorPlanSummary {
    pub file_count: usize,
    pub definition_count: usize,
    pub reference_count: usize,
    pub call_count: usize,
    pub inbound_edge_count: usize,
    pub outbound_edge_count: usize,
    pub non_call_reference_count: usize,
    pub signature_mismatch_count: usize,
    pub safe_to_automate: bool,
}

#[derive(Debug, Clone)]
pub struct RefactorPlanGate {
    pub level: RefactorRiskLevel,
    pub code: &'static str,
    pub message: String,
    pub count: usize,
    pub blocks_automation: bool,
}

#[derive(Debug, Clone)]
pub struct RefactorPlanStep {
    pub order: usize,
    pub action: &'static str,
    pub rationale: String,
    pub command: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RefactorPlanPolicy {
    pub fail_on_blocking_gate: bool,
    pub require_definitions: Option<usize>,
    pub require_references: Option<usize>,
    pub blocking_gate_count: usize,
    pub definition_count: usize,
    pub reference_count: usize,
    pub passed: bool,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct RefactorPlanPolicyRequest {
    pub fail_on_blocking_gate: bool,
    pub require_definitions: Option<usize>,
    pub require_references: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RefactorVerificationCheck {
    pub code: &'static str,
    pub level: RefactorRiskLevel,
    pub passed: bool,
    pub message: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct RefactorVerificationRequest<'a> {
    pub operation: RefactorOperation,
    pub phase: VerificationPhase,
    pub symbol: &'a str,
    pub new_symbol: Option<&'a str>,
    pub before: RefactorPlanSummary,
    pub after: Option<RefactorPlanSummary>,
}

#[derive(Debug, Clone)]
pub struct RawRefactorRisk {
    pub level: RefactorRiskLevel,
    pub code: &'static str,
    pub message: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct RefactorPlanRequest<'a> {
    pub operation: RefactorOperation,
    pub symbol: &'a str,
    pub files: &'a [PathBuf],
    pub summary: RefactorPlanSummary,
    pub policy: RefactorPlanPolicyRequest,
    pub risks: Vec<RawRefactorRisk>,
}

#[derive(Debug, Clone)]
pub struct RefactorPlanDecision {
    pub gates: Vec<RefactorPlanGate>,
    pub steps: Vec<RefactorPlanStep>,
    pub policy: RefactorPlanPolicy,
}
