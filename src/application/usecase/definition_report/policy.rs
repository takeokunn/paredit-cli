use super::references::{
    unused_definition_actionable_candidate_count, unused_definition_candidate_count,
};
use super::types::{UnusedDefinitionFile, UnusedDefinitionPolicy, UnusedDefinitionPolicyOptions};

pub fn evaluate_unused_definition_policy(
    options: UnusedDefinitionPolicyOptions,
    reports: &[UnusedDefinitionFile],
) -> UnusedDefinitionPolicy {
    let definition_count = reports
        .iter()
        .map(|report| report.definitions.len())
        .sum::<usize>();
    let candidate_count = unused_definition_candidate_count(reports);
    // Gates run against the actionable (bulk-removable-category) subset, not
    // the raw candidate_count: `Test`/`Package`/`Struct`/... candidates are
    // normally unreferenced by symbol by design (see
    // `DefinitionCategory::is_bulk_removable`), so gating on the raw count
    // would trip `--fail-on-unused` on any codebase with an ordinary test
    // suite.
    let actionable_candidate_count = unused_definition_actionable_candidate_count(reports);
    let mut violations = Vec::new();

    if options.fail_on_unused && actionable_candidate_count > 0 {
        violations.push(format!(
            "actionable_candidate_count {actionable_candidate_count} exceeds 0"
        ));
    }
    if let Some(required) = options.require_unused_definitions {
        if actionable_candidate_count < required {
            violations.push(format!(
                "actionable_candidate_count {actionable_candidate_count} is below required {required}"
            ));
        }
    }

    UnusedDefinitionPolicy {
        fail_on_unused: options.fail_on_unused,
        require_unused_definitions: options.require_unused_definitions,
        definition_count,
        candidate_count,
        actionable_candidate_count,
        passed: violations.is_empty(),
        violations,
    }
}
