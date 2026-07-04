use std::path::PathBuf;

use clap::Args;

use crate::domain::sexpr::SymbolName;
use crate::presentation::cli::{DialectArg, OutputFormat};

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct SymbolQueryArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Exact symbol atom to find.
    #[arg(long)]
    pub(super) symbol: SymbolName,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct SymbolReportArgs {
    /// Files to scan.
    #[arg(required = true)]
    pub(super) files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Exact symbol atom to report.
    #[arg(long)]
    pub(super) symbol: SymbolName,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}
