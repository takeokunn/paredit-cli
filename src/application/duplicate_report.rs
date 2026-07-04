//! Duplicate form analysis for replacement planning.

mod batches;
mod collect;
mod reports;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

pub use batches::collect_replacement_plan_batches;
pub use collect::collect_duplicate_candidates;
pub use reports::build_duplicate_shape_reports;
pub use types::{
    DuplicateCandidateGroups, DuplicateFormReport, DuplicateShapeReport, ReplacementPlanBatch,
};
