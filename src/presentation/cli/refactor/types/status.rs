use super::super::super::*;
use super::check::{RefactorCheckFileResult, RefactorCheckSummary};
use super::manifest::RefactorApplyManifestHeader;
use super::root::RefactorRootReport;

#[derive(Debug)]
pub(in crate::presentation::cli) struct RefactorStatusResult {
    pub(in crate::presentation::cli) manifest: RefactorApplyManifestHeader,
    pub(in crate::presentation::cli) root: RefactorRootReport,
    pub(in crate::presentation::cli) manifest_policy_passed: bool,
    pub(in crate::presentation::cli) manifest_outputs_parse: bool,
    pub(in crate::presentation::cli) status: RefactorStatusKind,
    pub(in crate::presentation::cli) next_action: RefactorStatusNextAction,
    pub(in crate::presentation::cli) blocked_reasons: Vec<RefactorStatusBlockedReason>,
    pub(in crate::presentation::cli) write_plan: Vec<RefactorStatusWriteTarget>,
    pub(in crate::presentation::cli) files: Vec<RefactorCheckFileResult>,
    pub(in crate::presentation::cli) summary: RefactorCheckSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::presentation::cli) enum RefactorStatusKind {
    Ready,
    Blocked,
}

impl RefactorStatusKind {
    pub(in crate::presentation::cli) fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::presentation::cli) enum RefactorStatusNextAction {
    RunDiffThenApplyWrite,
    RegeneratePreview,
    FixManifestOrParser,
}

impl RefactorStatusNextAction {
    pub(in crate::presentation::cli) fn label(self) -> &'static str {
        match self {
            Self::RunDiffThenApplyWrite => "run_refactor_diff_then_refactor_apply_write",
            Self::RegeneratePreview => "regenerate_refactor_preview",
            Self::FixManifestOrParser => "fix_manifest_or_parser",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::presentation::cli) enum RefactorStatusBlockedReason {
    ManifestPolicyFailed,
    ManifestOutputsDoNotParse,
    StaleFiles,
    OutputHashMismatches,
    ParseErrors,
    ManifestFlagMismatches,
}

impl RefactorStatusBlockedReason {
    pub(in crate::presentation::cli) fn label(self) -> &'static str {
        match self {
            Self::ManifestPolicyFailed => "manifest_policy_failed",
            Self::ManifestOutputsDoNotParse => "manifest_outputs_do_not_parse",
            Self::StaleFiles => "stale_files",
            Self::OutputHashMismatches => "output_hash_mismatches",
            Self::ParseErrors => "parse_errors",
            Self::ManifestFlagMismatches => "manifest_flag_mismatches",
        }
    }
}

#[derive(Debug)]
pub(in crate::presentation::cli) struct RefactorStatusWriteTarget {
    pub(in crate::presentation::cli) path: PathBuf,
    pub(in crate::presentation::cli) edit_count: usize,
    pub(in crate::presentation::cli) input_hash: String,
    pub(in crate::presentation::cli) output_hash: String,
}
