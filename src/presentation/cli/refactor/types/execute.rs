use super::preview::RefactorPreview;
use super::verification::RefactorVerification;
use crate::application::refactor::execute::{
    RefactorExecuteDecision, RefactorExecuteDecisionStatus, RefactorExecuteStep,
    RefactorExecuteStepStatus,
};

#[derive(Debug)]
pub(in crate::presentation::cli) struct WorkspaceRefactorExecute {
    pub(in crate::presentation::cli) preview: RefactorPreview,
    pub(in crate::presentation::cli) preflight_decision: RefactorExecuteDecision,
    pub(in crate::presentation::cli) execute_decision: RefactorExecuteDecision,
    pub(in crate::presentation::cli) outcome: WorkspaceRefactorExecuteOutcome,
    pub(in crate::presentation::cli) pre_verification: Option<RefactorVerification>,
    pub(in crate::presentation::cli) post_verification: Option<RefactorVerification>,
}

#[derive(Debug)]
pub(in crate::presentation::cli) struct WorkspaceRefactorExecuteOutcome {
    pub(in crate::presentation::cli) status: WorkspaceRefactorExecuteOutcomeStatus,
    pub(in crate::presentation::cli) write_applied: bool,
    pub(in crate::presentation::cli) post_verification_passed: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::presentation::cli) struct WorkspaceRefactorExecuteOutcomeSummary {
    pub(in crate::presentation::cli) passed_step_count: usize,
    pub(in crate::presentation::cli) failed_step_count: usize,
    pub(in crate::presentation::cli) skipped_step_count: usize,
    pub(in crate::presentation::cli) scheduled_step_count: usize,
    pub(in crate::presentation::cli) write_applied: bool,
    pub(in crate::presentation::cli) post_verification_passed: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::presentation::cli) enum WorkspaceRefactorExecuteOutcomeStatus {
    BlockedByPolicy,
    RefusedUnparsableOutput,
    BlockedByPreVerification,
    DryRunReady,
    WriteApplied,
    PostVerificationFailed,
}

impl WorkspaceRefactorExecuteOutcome {
    pub(in crate::presentation::cli) fn from_decision(
        decision: RefactorExecuteDecision,
        post_verification_passed: Option<bool>,
    ) -> Self {
        match decision.status {
            RefactorExecuteDecisionStatus::BlockedByPolicy => Self {
                status: WorkspaceRefactorExecuteOutcomeStatus::BlockedByPolicy,
                write_applied: false,
                post_verification_passed: None,
            },
            RefactorExecuteDecisionStatus::RefusedUnparsableOutput => Self {
                status: WorkspaceRefactorExecuteOutcomeStatus::RefusedUnparsableOutput,
                write_applied: false,
                post_verification_passed: None,
            },
            RefactorExecuteDecisionStatus::BlockedByPreVerification => Self {
                status: WorkspaceRefactorExecuteOutcomeStatus::BlockedByPreVerification,
                write_applied: false,
                post_verification_passed: None,
            },
            RefactorExecuteDecisionStatus::DryRunReady => Self {
                status: WorkspaceRefactorExecuteOutcomeStatus::DryRunReady,
                write_applied: false,
                post_verification_passed: None,
            },
            RefactorExecuteDecisionStatus::ReadyToWrite => {
                let post_passed = post_verification_passed.unwrap_or(false);
                Self {
                    status: if post_passed {
                        WorkspaceRefactorExecuteOutcomeStatus::WriteApplied
                    } else {
                        WorkspaceRefactorExecuteOutcomeStatus::PostVerificationFailed
                    },
                    write_applied: true,
                    post_verification_passed: Some(post_passed),
                }
            }
        }
    }

    pub(in crate::presentation::cli) fn steps(&self) -> Vec<RefactorExecuteStep> {
        use RefactorExecuteStepStatus::{Failed, Passed, Scheduled, Skipped};
        use WorkspaceRefactorExecuteOutcomeStatus::{
            BlockedByPolicy, BlockedByPreVerification, DryRunReady, PostVerificationFailed,
            RefusedUnparsableOutput, WriteApplied,
        };

        let status = self.status;
        let preview_policy = if matches!(status, BlockedByPolicy) {
            Failed
        } else {
            Passed
        };
        let output_parse = match status {
            BlockedByPolicy => Skipped,
            RefusedUnparsableOutput => Failed,
            BlockedByPreVerification | DryRunReady | WriteApplied | PostVerificationFailed => {
                Passed
            }
        };
        let pre_verification = match status {
            BlockedByPolicy | RefusedUnparsableOutput => Skipped,
            BlockedByPreVerification => Failed,
            DryRunReady | WriteApplied | PostVerificationFailed => Passed,
        };
        let apply_preview = match status {
            BlockedByPolicy | RefusedUnparsableOutput | BlockedByPreVerification => Skipped,
            DryRunReady => Scheduled,
            WriteApplied | PostVerificationFailed => Passed,
        };
        let post_verification = match status {
            WriteApplied => Passed,
            PostVerificationFailed => Failed,
            BlockedByPolicy | RefusedUnparsableOutput | BlockedByPreVerification | DryRunReady => {
                Skipped
            }
        };

        vec![
            RefactorExecuteStep {
                name: "preview-policy",
                status: preview_policy,
            },
            RefactorExecuteStep {
                name: "write-output-parse",
                status: output_parse,
            },
            RefactorExecuteStep {
                name: "pre-verification",
                status: pre_verification,
            },
            RefactorExecuteStep {
                name: "apply-preview",
                status: apply_preview,
            },
            RefactorExecuteStep {
                name: "post-verification",
                status: post_verification,
            },
        ]
    }

    pub(in crate::presentation::cli) fn summary(&self) -> WorkspaceRefactorExecuteOutcomeSummary {
        let mut summary = WorkspaceRefactorExecuteOutcomeSummary {
            passed_step_count: 0,
            failed_step_count: 0,
            skipped_step_count: 0,
            scheduled_step_count: 0,
            write_applied: self.write_applied,
            post_verification_passed: self.post_verification_passed,
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
}

impl WorkspaceRefactorExecuteOutcomeStatus {
    pub(in crate::presentation::cli) fn label(self) -> &'static str {
        match self {
            Self::BlockedByPolicy => "blocked-by-policy",
            Self::RefusedUnparsableOutput => "refused-unparsable-output",
            Self::BlockedByPreVerification => "blocked-by-pre-verification",
            Self::DryRunReady => "dry-run-ready",
            Self::WriteApplied => "write-applied",
            Self::PostVerificationFailed => "post-verification-failed",
        }
    }

    pub(in crate::presentation::cli) fn reason(self) -> &'static str {
        match self {
            Self::BlockedByPolicy => "preview-policy-failed",
            Self::RefusedUnparsableOutput => "rewritten-output-did-not-parse",
            Self::BlockedByPreVerification => "pre-verification-failed",
            Self::DryRunReady => "all-dry-run-gates-passed",
            Self::WriteApplied => "write-and-post-verification-passed",
            Self::PostVerificationFailed => "post-verification-failed",
        }
    }

    pub(in crate::presentation::cli) fn next_action(self) -> &'static str {
        match self {
            Self::BlockedByPolicy => "review-policy-violations",
            Self::RefusedUnparsableOutput => "inspect-preview-parse-errors",
            Self::BlockedByPreVerification => "review-pre-verification-checks",
            Self::DryRunReady => "review-preview-or-rerun-with-write",
            Self::WriteApplied => "review-written-files",
            Self::PostVerificationFailed => "review-post-verification-checks",
        }
    }
}
