use std::path::PathBuf;

use crate::domain::refactor_plan::{
    RawRefactorRisk, RefactorOperation, RefactorPlanGate, RefactorPlanPolicy,
    RefactorPlanPolicyOptions, RefactorPlanRiskSummary, RefactorPlanSummary,
    RefactorPlanTargetKind,
};

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
pub struct RefactorPlanRequest<'a> {
    pub operation: RefactorOperation,
    pub symbol: &'a str,
    pub files: &'a [PathBuf],
    pub target_kind: RefactorPlanTargetKind,
    pub summary: RefactorPlanSummary,
    pub policy: RefactorPlanPolicyOptions,
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
