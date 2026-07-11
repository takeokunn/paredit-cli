mod collect;
mod reports;
mod types;

pub use collect::collect_similarity_candidates;
pub use reports::build_similarity_pairs;
pub use types::{
    SimilarityCandidate, SimilarityFormReport, SimilarityOverlapPolicy, SimilarityPairReport,
    SimilarityReport, SimilarityReportSummary,
};

#[cfg(test)]
mod tests;
