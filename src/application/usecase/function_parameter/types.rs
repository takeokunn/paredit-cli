use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Path, SymbolName};

#[derive(Debug, Clone)]
pub struct AddFunctionParameterRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub definition_path: Path,
    pub name: SymbolName,
    pub argument: String,
    pub call_paths: Vec<Path>,
    pub all_calls: bool,
    pub insert: FunctionParameterInsert,
}

#[derive(Debug, Clone)]
pub struct MoveFunctionParameterRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub definition_path: Path,
    pub name: SymbolName,
    pub to_index: usize,
    pub call_paths: Vec<Path>,
    pub all_calls: bool,
}

#[derive(Debug, Clone)]
pub struct RemoveFunctionParameterRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub definition_path: Path,
    pub name: SymbolName,
    pub call_paths: Vec<Path>,
    pub all_calls: bool,
    pub allow_missing_argument: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionParameterInsert {
    Start,
    End,
}

impl FunctionParameterInsert {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AddFunctionParameterPlan {
    pub dialect: Dialect,
    pub definition_path: Path,
    pub call_paths: Vec<Path>,
    pub all_calls: bool,
    pub definition_span: ByteSpan,
    pub parameter_list_span: ByteSpan,
    pub call_spans: Vec<ByteSpan>,
    pub function_name: SymbolName,
    pub parameter_name: SymbolName,
    pub argument: String,
    pub insert: FunctionParameterInsert,
    pub rewritten: String,
    pub changed: bool,
}

#[derive(Debug, Clone)]
pub struct MoveFunctionParameterPlan {
    pub dialect: Dialect,
    pub definition_path: Path,
    pub call_paths: Vec<Path>,
    pub all_calls: bool,
    pub definition_span: ByteSpan,
    pub parameter_list_span: ByteSpan,
    pub call_spans: Vec<ByteSpan>,
    pub function_name: SymbolName,
    pub parameter_name: SymbolName,
    pub from_index: usize,
    pub to_index: usize,
    pub moved_arguments: Vec<String>,
    pub rewritten: String,
    pub changed: bool,
}

#[derive(Debug, Clone)]
pub struct RemoveFunctionParameterPlan {
    pub dialect: Dialect,
    pub definition_path: Path,
    pub call_paths: Vec<Path>,
    pub all_calls: bool,
    pub definition_span: ByteSpan,
    pub parameter_list_span: ByteSpan,
    pub call_spans: Vec<ByteSpan>,
    pub function_name: SymbolName,
    pub parameter_name: SymbolName,
    pub parameter_index: usize,
    pub removed_arguments: Vec<Option<String>>,
    pub rewritten: String,
    pub changed: bool,
}
