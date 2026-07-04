use std::collections::BTreeMap;

use crate::domain::dialect::Dialect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceFileStatus {
    Parsed,
    ParseError(String),
}

impl WorkspaceFileStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Parsed => "parsed",
            Self::ParseError(_) => "parse-error",
        }
    }

    pub fn is_parsed(&self) -> bool {
        matches!(self, Self::Parsed)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorkspaceFileMetrics<'a> {
    pub dialect: Dialect,
    pub status: &'a WorkspaceFileStatus,
    pub byte_count: usize,
    pub top_level_form_count: usize,
    pub atom_count: usize,
    pub definition_count: usize,
    pub call_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceReportSummary {
    pub file_count: usize,
    pub parsed_count: usize,
    pub parse_error_count: usize,
    pub byte_count: usize,
    pub top_level_form_count: usize,
    pub atom_count: usize,
    pub definition_count: usize,
    pub call_count: usize,
    pub dialect_counts: BTreeMap<&'static str, usize>,
    pub status_counts: BTreeMap<&'static str, usize>,
}
