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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityComparisonScope {
    All,
    SameFile,
    CrossFile,
}

impl SimilarityComparisonScope {
    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::SameFile => "same-file",
            Self::CrossFile => "cross-file",
        }
    }
}

impl FromStr for SimilarityComparisonScope {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "all" => Ok(Self::All),
            "same-file" => Ok(Self::SameFile),
            "cross-file" => Ok(Self::CrossFile),
            _ => Err(format!("unknown comparison scope: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimilarityFormScope {
    All,
    TopLevel,
}

impl SimilarityFormScope {
    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::TopLevel => "top-level",
        }
    }
}

impl FromStr for SimilarityFormScope {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "all" => Ok(Self::All),
            "top-level" => Ok(Self::TopLevel),
            _ => Err(format!("unknown form scope: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityReportOptions {
    pub threshold: f64,
    pub min_node_count: usize,
    pub min_line_span: usize,
    pub comparison_scope: SimilarityComparisonScope,
    pub form_scope: SimilarityFormScope,
    pub overlap_policy: SimilarityOverlapPolicy,
    pub max_candidates: Option<usize>,
    pub max_comparisons: Option<usize>,
    pub max_results: Option<usize>,
}

impl Default for SimilarityReportOptions {
    fn default() -> Self {
        Self {
            threshold: 0.87,
            min_node_count: 4,
            min_line_span: 1,
            comparison_scope: SimilarityComparisonScope::All,
            form_scope: SimilarityFormScope::All,
            overlap_policy: SimilarityOverlapPolicy::Maximal,
            max_candidates: None,
            max_comparisons: None,
            max_results: None,
        }
    }
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
