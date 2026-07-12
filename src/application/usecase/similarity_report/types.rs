use std::path::PathBuf;

use crate::application::form_similarity::StructuralTree;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::ByteSpan;

#[allow(unused_imports)]
pub use crate::domain::similarity_report::{
    SimilarityComparisonScope, SimilarityFormScope, SimilarityOverlapPolicy,
    SimilarityReportOptions, SimilarityReportOptionsError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarityFormReport {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub form_path: String,
    pub span: ByteSpan,
    pub node_count: usize,
    pub head: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarityCandidate {
    pub form: SimilarityFormReport,
    pub tree: StructuralTree,
    pub comparison_head: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityPairReport {
    pub similarity: f64,
    pub score: f64,
    pub left: SimilarityFormReport,
    pub right: SimilarityFormReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarityReportSummary {
    pub candidate_limit_reached: bool,
    pub omitted_candidates: usize,
    pub possible_pairs: usize,
    pub evaluated_pairs: usize,
    pub pruned_by_size: usize,
    pub comparison_limit_reached: bool,
    pub unprocessed_pairs: usize,
    pub matched_pairs: usize,
    pub suppressed_pairs: usize,
    pub reported_pairs: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityReport {
    pub summary: SimilarityReportSummary,
    pub pairs: Vec<SimilarityPairReport>,
}
