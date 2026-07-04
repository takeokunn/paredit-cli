use std::path::PathBuf;

use clap::Args;

use super::super::{DialectArg, OutputFormat};
use crate::domain::sexpr::{Path, SymbolName};

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct RenameSymbolArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Exact source symbol atom.
    #[arg(long)]
    pub(super) from: SymbolName,
    /// Exact replacement symbol atom.
    #[arg(long)]
    pub(super) to: SymbolName,
    /// Print occurrence metadata instead of rewritten source.
    #[arg(long)]
    pub(super) plan: bool,
    /// Output format for --plan.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct RenameInFormArgs {
    /// Input file. Required when --write is used; reads stdin otherwise.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Exact source symbol atom.
    #[arg(long)]
    pub(super) from: SymbolName,
    /// Exact replacement symbol atom.
    #[arg(long)]
    pub(super) to: SymbolName,
    /// Select the refactor scope by child index path, for example 0.3.
    #[arg(long, conflicts_with = "at")]
    pub(super) path: Option<Path>,
    /// Select the smallest refactor scope containing byte offset.
    #[arg(long, conflicts_with = "path")]
    pub(super) at: Option<usize>,
    /// Rewrite the input file in place. Without this flag, only prints a plan.
    #[arg(long)]
    pub(super) write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct RenameBindingArgs {
    /// Input file. Required when --write is used; reads stdin otherwise.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Select the let form by child index path, for example 0.3.
    #[arg(long, conflicts_with = "at")]
    pub(super) path: Option<Path>,
    /// Select the smallest let form containing byte offset.
    #[arg(long, conflicts_with = "path")]
    pub(super) at: Option<usize>,
    /// Existing binding name.
    #[arg(long)]
    pub(super) from: SymbolName,
    /// Replacement binding name.
    #[arg(long)]
    pub(super) to: SymbolName,
    /// Rewrite the input file in place. Without this flag, only prints a plan.
    #[arg(long)]
    pub(super) write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct RenameSymbolsArgs {
    /// Files to scan and optionally rewrite.
    #[arg(required = true)]
    pub(super) files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Exact source symbol atom.
    #[arg(long)]
    pub(super) from: SymbolName,
    /// Exact replacement symbol atom.
    #[arg(long)]
    pub(super) to: SymbolName,
    /// Rewrite changed files in place. Without this flag, only prints a plan.
    #[arg(long)]
    pub(super) write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct RenameFunctionArgs {
    /// Files to scan and optionally rewrite.
    #[arg(required = true)]
    pub(super) files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Existing callable definition name.
    #[arg(long)]
    pub(super) from: SymbolName,
    /// Replacement callable definition name.
    #[arg(long)]
    pub(super) to: SymbolName,
    /// Rewrite changed files in place. Without this flag, only prints a plan.
    #[arg(long)]
    pub(super) write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct RenameMacroletArgs {
    /// Files to scan and optionally rewrite.
    #[arg(required = true)]
    pub(super) files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Existing macrolet or compiler-macrolet binding name.
    #[arg(long)]
    pub(super) from: SymbolName,
    /// Replacement macrolet or compiler-macrolet binding name.
    #[arg(long)]
    pub(super) to: SymbolName,
    /// Rewrite changed files in place. Without this flag, only prints a plan.
    #[arg(long)]
    pub(super) write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct WrapFunctionCallsArgs {
    /// Files to scan and optionally rewrite.
    #[arg(required = true)]
    pub(super) files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Existing callable head to wrap.
    #[arg(long)]
    pub(super) function: SymbolName,
    /// Wrapper callable or macro inserted around each selected call.
    #[arg(long)]
    pub(super) wrapper: SymbolName,
    /// Wrapper form template containing exactly one "_" placeholder for the original call.
    #[arg(long = "wrapper-template")]
    pub(super) wrapper_template: Option<String>,
    /// Wrap all matching call sites.
    #[arg(long)]
    pub(super) all_calls: bool,
    /// Wrap only the call sites at these expression paths.
    #[arg(long = "call-path")]
    pub(super) call_paths: Vec<Path>,
    /// Rewrite changed files in place. Without this flag, only prints a plan.
    #[arg(long)]
    pub(super) write: bool,
    /// Fail if no selected call site changes.
    #[arg(long)]
    pub(super) fail_on_no_change: bool,
    /// Require at least this many call-site rewrites.
    #[arg(long)]
    pub(super) require_calls: Option<usize>,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct ReplaceFunctionCallsArgs {
    /// Files to scan and optionally rewrite.
    #[arg(required = true)]
    pub(super) files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Existing callable call head.
    #[arg(long)]
    pub(super) from: SymbolName,
    /// Replacement callable call head.
    #[arg(long)]
    pub(super) to: SymbolName,
    /// Replace all matching call sites.
    #[arg(long)]
    pub(super) all_calls: bool,
    /// Replace only the call sites at these expression paths.
    #[arg(long = "call-path")]
    pub(super) call_paths: Vec<Path>,
    /// Rewrite changed files in place. Without this flag, only prints a plan.
    #[arg(long)]
    pub(super) write: bool,
    /// Fail if no selected call site changes.
    #[arg(long)]
    pub(super) fail_on_no_change: bool,
    /// Require at least this many call-site rewrites.
    #[arg(long)]
    pub(super) require_calls: Option<usize>,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct UnwrapFunctionCallsArgs {
    /// Files to scan and optionally rewrite.
    #[arg(required = true)]
    pub(super) files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Existing callable head inside the wrapper.
    #[arg(long)]
    pub(super) function: SymbolName,
    /// Wrapper callable or macro removed around each selected call.
    #[arg(long)]
    pub(super) wrapper: SymbolName,
    /// Unwrap all matching unary wrapper call sites.
    #[arg(long)]
    pub(super) all_calls: bool,
    /// Unwrap only the wrapper call sites at these expression paths.
    #[arg(long = "call-path")]
    pub(super) call_paths: Vec<Path>,
    /// Rewrite changed files in place. Without this flag, only prints a plan.
    #[arg(long)]
    pub(super) write: bool,
    /// Fail if no selected call site changes.
    #[arg(long)]
    pub(super) fail_on_no_change: bool,
    /// Require at least this many call-site rewrites.
    #[arg(long)]
    pub(super) require_calls: Option<usize>,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}
