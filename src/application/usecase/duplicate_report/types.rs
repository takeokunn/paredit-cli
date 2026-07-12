use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::domain::dialect::Dialect;
use crate::domain::form_shape::FormShape;
use crate::domain::sexpr::{ByteSpan, Path};

#[derive(Debug, Clone)]
pub struct DuplicateFormReport {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub form_path: Path,
    pub span: ByteSpan,
    pub node_count: usize,
    pub head: Option<String>,
    pub text: String,
}

#[derive(Debug)]
pub struct DuplicateShapeReport {
    pub shape: FormShape,
    pub count: usize,
    pub forms: Vec<DuplicateFormReport>,
}

#[derive(Debug)]
pub struct ReplacementPlanBatch {
    pub file: PathBuf,
    pub dialect: Dialect,
    pub shape: FormShape,
    pub replacement: String,
    pub keep_first: bool,
    pub forms: Vec<DuplicateFormReport>,
}

pub type DuplicateCandidateGroups = BTreeMap<FormShape, Vec<DuplicateFormReport>>;
