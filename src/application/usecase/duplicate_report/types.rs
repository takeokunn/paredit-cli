use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::ByteSpan;

#[derive(Debug, Clone)]
pub struct DuplicateFormReport {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub form_path: String,
    pub span: ByteSpan,
    pub node_count: usize,
    pub head: Option<String>,
    pub text: String,
}

#[derive(Debug)]
pub struct DuplicateShapeReport {
    pub shape: String,
    pub count: usize,
    pub forms: Vec<DuplicateFormReport>,
}

#[derive(Debug)]
pub struct ReplacementPlanBatch {
    pub file: PathBuf,
    pub dialect: Dialect,
    pub shape: String,
    pub replacement: String,
    pub keep_first: bool,
    pub forms: Vec<DuplicateFormReport>,
}

pub type DuplicateCandidateGroups = BTreeMap<String, Vec<DuplicateFormReport>>;
