use std::path::PathBuf;

use clap::Args;

use crate::domain::sexpr::{Path, SymbolName};
use crate::presentation::cli::args::{DialectArg, OutputFormat, ParameterInsert};

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct AddFunctionParameterArgs {
    #[arg(short, long)]
    pub(in crate::presentation::cli::function_parameter) file: Option<PathBuf>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) dialect: Option<DialectArg>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) definition_path: Path,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) name: SymbolName,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) argument: String,
    #[arg(long = "call-path")]
    pub(in crate::presentation::cli::function_parameter) call_paths: Vec<Path>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) all_calls: bool,
    #[arg(long, value_enum, default_value_t = ParameterInsert::End)]
    pub(in crate::presentation::cli::function_parameter) insert: ParameterInsert,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) write: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(in crate::presentation::cli::function_parameter) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct MoveFunctionParameterArgs {
    #[arg(short, long)]
    pub(in crate::presentation::cli::function_parameter) file: Option<PathBuf>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) dialect: Option<DialectArg>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) definition_path: Path,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) name: SymbolName,
    #[arg(long = "to-index")]
    pub(in crate::presentation::cli::function_parameter) to_index: usize,
    #[arg(long = "call-path")]
    pub(in crate::presentation::cli::function_parameter) call_paths: Vec<Path>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) all_calls: bool,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) write: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(in crate::presentation::cli::function_parameter) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct SwapFunctionParametersArgs {
    #[arg(short, long)]
    pub(in crate::presentation::cli::function_parameter) file: Option<PathBuf>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) dialect: Option<DialectArg>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) definition_path: Path,
    #[arg(long = "left-name")]
    pub(in crate::presentation::cli::function_parameter) left_name: SymbolName,
    #[arg(long = "right-name")]
    pub(in crate::presentation::cli::function_parameter) right_name: SymbolName,
    #[arg(long = "call-path")]
    pub(in crate::presentation::cli::function_parameter) call_paths: Vec<Path>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) all_calls: bool,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) write: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(in crate::presentation::cli::function_parameter) output: OutputFormat,
}

#[derive(Debug, Args)]
pub(in crate::presentation::cli) struct RemoveFunctionParameterArgs {
    #[arg(short, long)]
    pub(in crate::presentation::cli::function_parameter) file: Option<PathBuf>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) dialect: Option<DialectArg>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) definition_path: Path,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) name: SymbolName,
    #[arg(long = "call-path")]
    pub(in crate::presentation::cli::function_parameter) call_paths: Vec<Path>,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) all_calls: bool,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) allow_missing_argument: bool,
    #[arg(long)]
    pub(in crate::presentation::cli::function_parameter) write: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub(in crate::presentation::cli::function_parameter) output: OutputFormat,
}
