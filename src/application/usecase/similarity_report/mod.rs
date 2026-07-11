mod collect;
mod reports;
mod types;

pub use collect::collect_similarity_candidates;
pub use reports::build_similarity_pairs;
pub use types::{
    SimilarityCandidate, SimilarityComparisonScope, SimilarityFormReport, SimilarityFormScope,
    SimilarityOverlapPolicy, SimilarityPairReport, SimilarityReport, SimilarityReportOptions,
    SimilarityReportSummary,
};

#[cfg(test)]
mod tests;
