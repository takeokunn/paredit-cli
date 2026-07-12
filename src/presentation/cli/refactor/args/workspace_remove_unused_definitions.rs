use std::path::PathBuf;

use clap::Args;

use super::super::super::OutputFormat;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  paredit refactor workspace-remove-unused-definitions --write --output json --exclude target .\n  paredit refactor workspace-remove-unused-definitions --include-hidden --include-generated --output json ."
)]
pub(in crate::presentation::cli) struct WorkspaceRemoveUnusedDefinitionsArgs {
    /// Files or directories to scan recursively.
    #[arg(required = true)]
    pub(in crate::presentation::cli) roots: Vec<PathBuf>,
    /// Include files whose extension does not identify a known Lisp dialect.
    #[arg(long)]
    pub(in crate::presentation::cli) include_unknown: bool,
    /// Include hidden directories and files.
    #[arg(long)]
    pub(in crate::presentation::cli) include_hidden: bool,
    /// Include generated or dependency directories such as target and node_modules.
    #[arg(long)]
    pub(in crate::presentation::cli) include_generated: bool,
    /// Exclude files or directories whose path starts with the given prefixes.
    #[arg(long)]
    pub(in crate::presentation::cli) exclude: Vec<PathBuf>,
    /// Maximum directory recursion depth from each root directory.
    #[arg(long)]
    pub(in crate::presentation::cli) max_depth: Option<usize>,
    /// Keep definitions that are marked protected.
    #[arg(long)]
    pub(in crate::presentation::cli) include_protected: bool,
    /// Keep definitions that are exported from a package.
    #[arg(long)]
    pub(in crate::presentation::cli) include_exported: bool,
    /// Write rewrites back to disk after the plan succeeds.
    #[arg(long)]
    pub(in crate::presentation::cli) write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(in crate::presentation::cli) output: OutputFormat,
}
