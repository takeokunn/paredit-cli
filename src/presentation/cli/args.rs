use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::application::usecase::extract_function::ExtractFunctionInsert;
use crate::application::usecase::function_parameter::FunctionParameterInsert;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::Path;

#[derive(Debug, Args)]
pub(super) struct InputArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(super) struct AnalyzeArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(super) struct FormReportArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Selected expression path, such as 0.2.1.
    #[arg(long, conflicts_with = "at")]
    pub(super) path: Option<Path>,
    /// Byte offset inside the selected expression.
    #[arg(long, conflicts_with = "path")]
    pub(super) at: Option<usize>,
    /// Include the selected source text in the report.
    #[arg(long)]
    pub(super) include_source: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(super) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(super) struct FormatArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    pub(super) dialect: Option<DialectArg>,
    /// Number of spaces per nesting level.
    #[arg(long, default_value_t = 2)]
    pub(super) indent: usize,
}

#[derive(Debug, Args)]
pub(super) struct TargetArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
    /// Select by child index path, for example 0.2.1.
    #[arg(long, conflicts_with = "at")]
    pub(super) path: Option<Path>,
    /// Select the smallest expression containing byte offset.
    #[arg(long, conflicts_with = "path")]
    pub(super) at: Option<usize>,
}

#[derive(Debug, Args)]
pub(super) struct ReplaceArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    pub(super) file: Option<PathBuf>,
    /// Select by child index path, for example 0.2.1.
    #[arg(long, conflicts_with = "at")]
    pub(super) path: Option<Path>,
    /// Select the smallest expression containing byte offset.
    #[arg(long, conflicts_with = "path")]
    pub(super) at: Option<usize>,
    /// Replacement S-expression text.
    #[arg(long)]
    pub(super) with: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(super) enum DialectArg {
    CommonLisp,
    EmacsLisp,
    Scheme,
    Clojure,
    Janet,
    Fennel,
    Unknown,
}

impl From<DialectArg> for Dialect {
    fn from(value: DialectArg) -> Self {
        match value {
            DialectArg::CommonLisp => Self::CommonLisp,
            DialectArg::EmacsLisp => Self::EmacsLisp,
            DialectArg::Scheme => Self::Scheme,
            DialectArg::Clojure => Self::Clojure,
            DialectArg::Janet => Self::Janet,
            DialectArg::Fennel => Self::Fennel,
            DialectArg::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(super) enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(super) enum MoveInsert {
    Append,
    Before,
    After,
}

impl MoveInsert {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Append => "append",
            Self::Before => "before",
            Self::After => "after",
        }
    }

    pub(super) fn into_extract_function_insert(self) -> ExtractFunctionInsert {
        match self {
            Self::Append => ExtractFunctionInsert::Append,
            Self::Before => ExtractFunctionInsert::Before,
            Self::After => ExtractFunctionInsert::After,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(super) enum ParameterInsert {
    Start,
    End,
}

impl ParameterInsert {
    pub(super) fn into_function_parameter_insert(self) -> FunctionParameterInsert {
        match self {
            Self::Start => FunctionParameterInsert::Start,
            Self::End => FunctionParameterInsert::End,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(super) enum ThreadStyleArg {
    First,
    Last,
}

#[derive(Debug)]
pub(super) struct SourceInput {
    pub(super) text: String,
    pub(super) file: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(super) enum ImpactRiskLevel {
    Info,
    Warning,
    Error,
}
