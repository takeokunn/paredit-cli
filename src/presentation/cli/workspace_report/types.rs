use std::path::PathBuf;

use crate::application::workspace_report::WorkspaceFileStatus;
use crate::domain::dialect::Dialect;

#[derive(Debug)]
pub(super) struct WorkspaceFileReport {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) status: WorkspaceFileStatus,
    pub(super) byte_count: usize,
    pub(super) top_level_form_count: usize,
    pub(super) atom_count: usize,
    pub(super) definition_count: usize,
    pub(super) call_count: usize,
    pub(super) package: Option<String>,
}
