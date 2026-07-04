use std::path::PathBuf;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::ByteSpan;

#[derive(Debug)]
pub(super) struct SymbolReportFile {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) occurrences: Vec<SymbolReportOccurrence>,
}

#[derive(Debug)]
pub(super) struct SymbolReportOccurrence {
    pub(super) path: String,
    pub(super) span: ByteSpan,
    pub(super) context: Option<SymbolOccurrenceContext>,
}

#[derive(Debug)]
pub(super) struct SymbolOccurrenceContext {
    pub(super) path: String,
    pub(super) span: ByteSpan,
    pub(super) head: Option<String>,
    pub(super) definition_like: bool,
}
