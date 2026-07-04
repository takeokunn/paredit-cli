use super::super::super::*;
use super::super::args::RefactorPreviewMode;
use super::plan::WorkspaceRefactorPlanDiscovery;

#[derive(Debug)]
pub(in crate::presentation::cli) struct RefactorPreview {
    pub(in crate::presentation::cli) workspace: Option<WorkspaceRefactorPlanDiscovery>,
    pub(in crate::presentation::cli) mode: RefactorPreviewMode,
    pub(in crate::presentation::cli) from: String,
    pub(in crate::presentation::cli) to: String,
    pub(in crate::presentation::cli) write_requested: bool,
    pub(in crate::presentation::cli) files: Vec<RefactorPreviewFile>,
    pub(in crate::presentation::cli) summary: RefactorPreviewSummary,
    pub(in crate::presentation::cli) policy: RefactorPreviewPolicy,
}

#[derive(Debug)]
pub(in crate::presentation::cli) struct RefactorPreviewFile {
    pub(in crate::presentation::cli) path: PathBuf,
    pub(in crate::presentation::cli) dialect: Dialect,
    pub(in crate::presentation::cli) changed: bool,
    pub(in crate::presentation::cli) written: bool,
    pub(in crate::presentation::cli) edit_count: usize,
    pub(in crate::presentation::cli) edits: Vec<RefactorPreviewEdit>,
    pub(in crate::presentation::cli) input_bytes: usize,
    pub(in crate::presentation::cli) output_bytes: usize,
    pub(in crate::presentation::cli) output_parse_ok: bool,
    pub(in crate::presentation::cli) input_hash: String,
    pub(in crate::presentation::cli) output_hash: String,
    pub(in crate::presentation::cli) preview: String,
    pub(in crate::presentation::cli) rewritten: String,
}
