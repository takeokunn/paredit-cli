use std::path::PathBuf;

use crate::application::usecase::call_report::CallReportItem;
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, SyntaxTree};

#[derive(Debug)]
pub struct SignatureReportSource {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub tree: SyntaxTree,
}

#[derive(Debug)]
pub struct SignatureReportFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub definitions: Vec<SignatureDefinitionItem>,
    pub calls: Vec<SignatureCallItem>,
}

#[derive(Debug, Clone)]
pub struct SignatureDefinitionItem {
    pub path: String,
    pub span: ByteSpan,
    pub head: String,
    pub name: Option<String>,
    pub category: DefinitionCategory,
    pub parameter_count: Option<usize>,
}

#[derive(Debug)]
pub struct SignatureCallItem {
    pub call: CallReportItem,
    pub expected_parameter_count: Option<usize>,
    pub status: SignatureCallStatus,
}

#[derive(Debug)]
pub struct SignatureReportPolicy {
    pub fail_on_mismatch: bool,
    pub require_definitions: Option<usize>,
    pub require_calls: Option<usize>,
    pub definition_count: usize,
    pub call_count: usize,
    pub mismatch_count: usize,
    pub passed: bool,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignatureCallStatus {
    Exact,
    MissingArguments,
    ExtraArguments,
    UnknownDefinition,
    AmbiguousDefinition,
}

impl SignatureCallStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::MissingArguments => "missing-arguments",
            Self::ExtraArguments => "extra-arguments",
            Self::UnknownDefinition => "unknown-definition",
            Self::AmbiguousDefinition => "ambiguous-definition",
        }
    }
}
