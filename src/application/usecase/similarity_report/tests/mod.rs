use std::path::PathBuf;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteOffset, ByteSpan, SyntaxTree};

use super::*;

fn candidates(file: &str, input: &str, min_node_count: usize) -> Vec<SimilarityCandidate> {
    let tree = SyntaxTree::parse(input).unwrap();
    let mut result = Vec::new();
    let options = SimilarityReportOptions {
        min_node_count,
        ..SimilarityReportOptions::default()
    };
    collect_similarity_candidates(
        &tree,
        input,
        std::path::Path::new(file),
        Dialect::CommonLisp,
        &options,
        &mut result,
    )
    .unwrap();
    result
}

fn build_similarity_pairs(
    candidates: Vec<SimilarityCandidate>,
    threshold: f64,
    overlap_policy: SimilarityOverlapPolicy,
    max_results: Option<usize>,
) -> SimilarityReport {
    super::reports::build_similarity_pairs(
        candidates,
        &SimilarityReportOptions {
            threshold,
            overlap_policy,
            max_results,
            ..SimilarityReportOptions::default()
        },
    )
}

fn report_form(path: &str, start: usize, end: usize) -> SimilarityFormReport {
    SimilarityFormReport {
        path: PathBuf::from(path),
        dialect: Dialect::CommonLisp,
        form_path: format!("{start}:{end}"),
        span: ByteSpan::new(ByteOffset::new(start), ByteOffset::new(end)),
        node_count: 1,
        head: None,
        text: String::new(),
    }
}

mod limits;
mod overlap;
mod scope;
