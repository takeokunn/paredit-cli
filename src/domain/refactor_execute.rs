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
pub struct RefactorExecuteDecisionSummary {
    pub passed_step_count: usize,
    pub failed_step_count: usize,
    pub skipped_step_count: usize,
    pub scheduled_step_count: usize,
    pub write_parse_refused: bool,
    pub run_pre_verification: bool,
    pub apply_preview: bool,
    pub run_post_verification: bool,
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
    pub fn summary(self) -> RefactorExecuteDecisionSummary {
        let mut summary = RefactorExecuteDecisionSummary {
            passed_step_count: 0,
            failed_step_count: 0,
            skipped_step_count: 0,
            scheduled_step_count: 0,
            write_parse_refused: self.write_parse_refused,
            run_pre_verification: self.run_pre_verification,
            apply_preview: self.apply_preview,
            run_post_verification: self.run_post_verification,
        };

        for step in self.steps() {
            match step.status {
                RefactorExecuteStepStatus::Passed => summary.passed_step_count += 1,
                RefactorExecuteStepStatus::Failed => summary.failed_step_count += 1,
                RefactorExecuteStepStatus::Skipped => summary.skipped_step_count += 1,
                RefactorExecuteStepStatus::Scheduled => summary.scheduled_step_count += 1,
            }
        }

        summary
    }

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

pub fn build_refactor_execute_decision(
    inputs: RefactorExecuteGateInputs,
) -> RefactorExecuteDecision {
    let write_parse_refused = inputs.write_requested && !inputs.outputs_parse;
    let run_pre_verification = inputs.policy_passed && !write_parse_refused;
    let apply_preview = run_pre_verification && inputs.preflight_passed;
    let run_post_verification = inputs.write_requested && apply_preview;
    let status = if !inputs.policy_passed {
        RefactorExecuteDecisionStatus::BlockedByPolicy
    } else if write_parse_refused {
        RefactorExecuteDecisionStatus::RefusedUnparsableOutput
    } else if !inputs.preflight_passed {
        RefactorExecuteDecisionStatus::BlockedByPreVerification
    } else if inputs.write_requested {
        RefactorExecuteDecisionStatus::ReadyToWrite
    } else {
        RefactorExecuteDecisionStatus::DryRunReady
    };

    RefactorExecuteDecision {
        status,
        write_parse_refused,
        run_pre_verification,
        apply_preview,
        run_post_verification,
    }
}
