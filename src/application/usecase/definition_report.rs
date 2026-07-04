//! Definition inventory and unused-definition analysis.

mod collect;
mod policy;
mod references;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

pub use collect::{
    build_definition_report, build_parsed_definition_file, collect_definition_forms,
};
pub use policy::evaluate_unused_definition_policy;
pub use references::{collect_unused_definition_candidates, unused_definition_candidate_count};
pub use syntax::{body_form_count, count_lambda_parameters, definition_name, lambda_list_index};
pub use types::{
    DefinitionReference, DefinitionReportFile, DefinitionReportItem, ParsedDefinitionFile,
    UnusedDefinitionFile, UnusedDefinitionItem, UnusedDefinitionPolicy,
    UnusedDefinitionPolicyOptions,
};
