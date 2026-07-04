use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WorkspaceDiscoveryOptions {
    pub roots: Vec<PathBuf>,
    pub include_unknown: bool,
    pub include_hidden: bool,
    pub include_generated: bool,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct WorkspaceDiscovery {
    pub files: Vec<PathBuf>,
    pub skipped_unknown_count: usize,
    pub skipped_hidden_count: usize,
    pub skipped_generated_count: usize,
    pub skipped_symlink_count: usize,
}
