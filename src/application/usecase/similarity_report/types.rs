use std::path::PathBuf;
use std::str::FromStr;

use crate::application::form_similarity::StructuralTree;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::ByteSpan;

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
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityPairReport {
    pub similarity: f64,
    pub score: f64,
    pub left: SimilarityFormReport,
    pub right: SimilarityFormReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityOverlapPolicy {
    Maximal,
    All,
}

impl SimilarityOverlapPolicy {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Maximal => "maximal",
            Self::All => "all",
        }
    }
}

impl FromStr for SimilarityOverlapPolicy {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "maximal" => Ok(Self::Maximal),
            "all" => Ok(Self::All),
            _ => Err(format!("unknown overlap policy: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarityReportSummary {
    pub possible_pairs: usize,
    pub evaluated_pairs: usize,
    pub pruned_by_size: usize,
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
