use std::path::PathBuf;

use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
pub use crate::domain::report_policy::UnusedDefinitionPolicyOptions;
use crate::domain::sexpr::{AtomOccurrence, ByteSpan};

#[derive(Debug)]
pub struct DefinitionReportFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub package: Option<String>,
    pub definitions: Vec<DefinitionReportItem>,
}

#[derive(Debug, Clone)]
pub struct DefinitionReportItem {
    pub path: String,
    pub span: ByteSpan,
    pub head: String,
    pub name: Option<String>,
    pub category: DefinitionCategory,
    pub parameter_count: Option<usize>,
    pub body_form_count: Option<usize>,
    pub package: Option<String>,
}

#[derive(Debug)]
pub struct ParsedDefinitionFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub package: Option<String>,
    pub definitions: Vec<DefinitionReportItem>,
    pub atoms: Vec<AtomOccurrence>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct DefinitionReference {
    pub file_index: usize,
    pub path: String,
    pub span: ByteSpan,
}

#[derive(Debug)]
pub struct UnusedDefinitionItem {
    pub definition: DefinitionReportItem,
    pub references: Vec<DefinitionReference>,
}

#[derive(Debug)]
pub struct UnusedDefinitionFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub package: Option<String>,
    pub definitions: Vec<UnusedDefinitionItem>,
}

#[derive(Debug)]
pub struct UnusedDefinitionPolicy {
    pub fail_on_unused: bool,
    pub require_unused_definitions: Option<usize>,
    pub definition_count: usize,
    pub candidate_count: usize,
    pub actionable_candidate_count: usize,
    pub passed: bool,
    pub violations: Vec<String>,
}
