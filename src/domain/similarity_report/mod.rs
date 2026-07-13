mod collect;
mod options;
mod reports;
mod types;

pub use collect::{collect_similarity_candidates, SimilarityCandidateCollectionError};
pub use options::{
    SimilarityComparisonScope, SimilarityFormScope, SimilarityOverlapPolicy,
    SimilarityReportOptions, SimilarityReportOptionsError,
};
pub use reports::{build_similarity_pairs, build_similarity_pairs_with_omissions};
pub use types::{
    SimilarityCandidate, SimilarityFormReport, SimilarityPairReport, SimilarityReport,
    SimilarityReportSummary,
};

#[cfg(test)]
mod tests;
