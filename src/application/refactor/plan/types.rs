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
pub enum RefactorPlanTargetKind {
    Callable,
    Macro,
    CompilerMacro,
    SetfExpander,
    SymbolMacro,
    Unknown,
}

impl RefactorPlanTargetKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Callable => "callable",
            Self::Macro => "macro",
            Self::CompilerMacro => "compiler_macro",
            Self::SetfExpander => "setf_expander",
            Self::SymbolMacro => "symbol_macro",
            Self::Unknown => "unknown",
        }
    }

    pub fn is_macro_like(self) -> bool {
        matches!(
            self,
            Self::Macro | Self::CompilerMacro | Self::SetfExpander | Self::SymbolMacro
        )
    }

    pub fn skips_signature_compatibility(self) -> bool {
        self.is_macro_like()
    }

    pub fn requires_call_coverage(self, operation: RefactorOperation) -> bool {
        match operation {
            RefactorOperation::Rename | RefactorOperation::Move => {
                !matches!(self, Self::SymbolMacro)
            }
            RefactorOperation::Signature => !self.skips_signature_compatibility(),
            RefactorOperation::Remove => true,
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

impl From<crate::domain::impact_report::ImpactRiskLevel> for RefactorRiskLevel {
    fn from(value: crate::domain::impact_report::ImpactRiskLevel) -> Self {
        match value {
            crate::domain::impact_report::ImpactRiskLevel::Info => Self::Info,
            crate::domain::impact_report::ImpactRiskLevel::Warning => Self::Warning,
            crate::domain::impact_report::ImpactRiskLevel::Error => Self::Error,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorPlanRiskSummary {
    pub highest_level: Option<RefactorRiskLevel>,
    pub info_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub blocking_count: usize,
    pub advisory_count: usize,
}

impl RefactorPlanRiskSummary {
    pub fn from_gates(gates: &[RefactorPlanGate]) -> Self {
        let mut summary = Self {
            highest_level: None,
            info_count: 0,
            warning_count: 0,
            error_count: 0,
            blocking_count: 0,
            advisory_count: 0,
        };

        for gate in gates {
            summary.highest_level = Some(match summary.highest_level {
                Some(level) => level.max(gate.level),
                None => gate.level,
            });

            match gate.level {
                RefactorRiskLevel::Info => summary.info_count += gate.count,
                RefactorRiskLevel::Warning => summary.warning_count += gate.count,
                RefactorRiskLevel::Error => summary.error_count += gate.count,
            }

            if gate.blocks_automation {
                summary.blocking_count += gate.count;
            } else {
                summary.advisory_count += gate.count;
            }
        }

        summary
    }
}

#[derive(Debug, Clone)]
pub struct RefactorPlanStep {
    pub order: usize,
    pub action: &'static str,
    pub rationale: String,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefactorPlanAutomationStatus {
    Ready,
    ManualReview,
    PolicyFailed,
}

impl RefactorPlanAutomationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::ManualReview => "manual_review",
            Self::PolicyFailed => "policy_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefactorPlanAutomationStepStatus {
    Passed,
    Failed,
    Skipped,
    Scheduled,
}

impl RefactorPlanAutomationStepStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
            Self::Scheduled => "scheduled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorPlanAutomationStep {
    pub name: &'static str,
    pub status: RefactorPlanAutomationStepStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactorPlanAutomationDecision {
    pub status: RefactorPlanAutomationStatus,
    pub reason: String,
    pub next_action: &'static str,
    pub safe_to_automate: bool,
    pub policy_passed: bool,
    pub blocking_gate_count: usize,
}

impl RefactorPlanAutomationDecision {
    pub fn steps(&self) -> [RefactorPlanAutomationStep; 3] {
        let policy_status = if self.policy_passed {
            RefactorPlanAutomationStepStatus::Passed
        } else {
            RefactorPlanAutomationStepStatus::Failed
        };
        let review_status = match self.status {
            RefactorPlanAutomationStatus::Ready => RefactorPlanAutomationStepStatus::Passed,
            RefactorPlanAutomationStatus::ManualReview => {
                RefactorPlanAutomationStepStatus::Scheduled
            }
            RefactorPlanAutomationStatus::PolicyFailed => RefactorPlanAutomationStepStatus::Skipped,
        };
        let apply_status = if self.safe_to_automate {
            RefactorPlanAutomationStepStatus::Scheduled
        } else {
            RefactorPlanAutomationStepStatus::Skipped
        };

        [
            RefactorPlanAutomationStep {
                name: "plan-policy",
                status: policy_status,
            },
            RefactorPlanAutomationStep {
                name: "manual-review-gates",
                status: review_status,
            },
            RefactorPlanAutomationStep {
                name: "apply-plan",
                status: apply_status,
            },
        ]
    }
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
    pub target_kind: RefactorPlanTargetKind,
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
    pub target_kind: RefactorPlanTargetKind,
    pub summary: RefactorPlanSummary,
    pub policy: RefactorPlanPolicyRequest,
    pub risks: Vec<RawRefactorRisk>,
}

#[derive(Debug, Clone)]
pub struct RefactorPlanDecision {
    pub gates: Vec<RefactorPlanGate>,
    pub risk_summary: RefactorPlanRiskSummary,
    pub steps: Vec<RefactorPlanStep>,
    pub policy: RefactorPlanPolicy,
    pub automation: RefactorPlanAutomationDecision,
}
