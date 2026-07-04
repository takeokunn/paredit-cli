#[derive(Debug, Clone, Copy)]
pub struct RefactorWriteCandidate {
    pub changed: bool,
    pub output_parse_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefactorWriteRefusal {
    UnparsableOutputs { count: usize },
}

impl RefactorWriteRefusal {
    pub fn label(&self) -> &'static str {
        match self {
            Self::UnparsableOutputs { .. } => "unparsable-outputs",
        }
    }

    pub fn reason(&self) -> &'static str {
        match self {
            Self::UnparsableOutputs { .. } => "rewritten-output-did-not-parse",
        }
    }

    pub fn next_action(&self) -> &'static str {
        match self {
            Self::UnparsableOutputs { .. } => "inspect-preview-parse-errors",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactorWritePlan {
    pub write_requested: bool,
    pub writable_indexes: Vec<usize>,
    pub refusal: Option<RefactorWriteRefusal>,
}

impl RefactorWritePlan {
    pub fn write_allowed(&self) -> bool {
        self.write_requested && self.refusal.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorExecuteGateInputs {
    pub write_requested: bool,
    pub policy_passed: bool,
    pub outputs_parse: bool,
    pub preflight_passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefactorExecuteDecisionStatus {
    BlockedByPolicy,
    RefusedUnparsableOutput,
    BlockedByPreVerification,
    ReadyToWrite,
    DryRunReady,
}

impl RefactorExecuteDecisionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::BlockedByPolicy => "blocked-by-policy",
            Self::RefusedUnparsableOutput => "refused-unparsable-output",
            Self::BlockedByPreVerification => "blocked-by-pre-verification",
            Self::ReadyToWrite => "ready-to-write",
            Self::DryRunReady => "dry-run-ready",
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            Self::BlockedByPolicy => "preview-policy-failed",
            Self::RefusedUnparsableOutput => "rewritten-output-did-not-parse",
            Self::BlockedByPreVerification => "pre-verification-failed",
            Self::ReadyToWrite => "all-execute-gates-passed",
            Self::DryRunReady => "all-dry-run-gates-passed",
        }
    }

    pub fn next_action(self) -> &'static str {
        match self {
            Self::BlockedByPolicy => "review-policy-violations",
            Self::RefusedUnparsableOutput => "inspect-preview-parse-errors",
            Self::BlockedByPreVerification => "review-pre-verification-checks",
            Self::ReadyToWrite => "write-preview-and-run-post-verification",
            Self::DryRunReady => "review-preview-or-rerun-with-write",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefactorExecuteStepStatus {
    Passed,
    Failed,
    Skipped,
    Scheduled,
}

impl RefactorExecuteStepStatus {
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
pub struct RefactorExecuteStep {
    pub name: &'static str,
    pub status: RefactorExecuteStepStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorExecuteDecision {
    pub status: RefactorExecuteDecisionStatus,
    pub write_parse_refused: bool,
    pub run_pre_verification: bool,
    pub apply_preview: bool,
    pub run_post_verification: bool,
}

impl RefactorExecuteDecision {
    pub fn steps(self) -> [RefactorExecuteStep; 5] {
        [
            RefactorExecuteStep {
                name: "preview-policy",
                status: if self.status == RefactorExecuteDecisionStatus::BlockedByPolicy {
                    RefactorExecuteStepStatus::Failed
                } else {
                    RefactorExecuteStepStatus::Passed
                },
            },
            RefactorExecuteStep {
                name: "write-output-parse",
                status: if self.status == RefactorExecuteDecisionStatus::BlockedByPolicy {
                    RefactorExecuteStepStatus::Skipped
                } else if self.write_parse_refused {
                    RefactorExecuteStepStatus::Failed
                } else {
                    RefactorExecuteStepStatus::Passed
                },
            },
            RefactorExecuteStep {
                name: "pre-verification",
                status: if self.status == RefactorExecuteDecisionStatus::BlockedByPreVerification {
                    RefactorExecuteStepStatus::Failed
                } else if self.run_pre_verification {
                    RefactorExecuteStepStatus::Passed
                } else {
                    RefactorExecuteStepStatus::Skipped
                },
            },
            RefactorExecuteStep {
                name: "apply-preview",
                status: if self.apply_preview {
                    RefactorExecuteStepStatus::Scheduled
                } else {
                    RefactorExecuteStepStatus::Skipped
                },
            },
            RefactorExecuteStep {
                name: "post-verification",
                status: if self.run_post_verification {
                    RefactorExecuteStepStatus::Scheduled
                } else {
                    RefactorExecuteStepStatus::Skipped
                },
            },
        ]
    }
}
