mod collect;
mod options;
mod reports;
mod types;

pub use collect::{SimilarityCandidateCollectionError, collect_similarity_candidates};
pub use options::{
    SimilarityComparisonScope, SimilarityFormScope, SimilarityOverlapPolicy,
    SimilarityReportOptions, SimilarityReportOptionsError,
};
pub use reports::{build_similarity_pairs, build_similarity_pairs_with_omissions};
pub use types::{
    SharedFormText, SimilarityCandidate, SimilarityFormReport, SimilarityPairReport,
    SimilarityReport, SimilarityReportSummary,
};

#[cfg(test)]
mod tests;
