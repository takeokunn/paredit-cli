use super::references::unused_definition_candidate_count;
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
    let mut violations = Vec::new();

    if options.fail_on_unused && candidate_count > 0 {
        violations.push(format!("candidate_count {candidate_count} exceeds 0"));
    }
    if let Some(required) = options.require_unused_definitions {
        if candidate_count < required {
            violations.push(format!(
                "candidate_count {candidate_count} is below required {required}"
            ));
        }
    }

    UnusedDefinitionPolicy {
        fail_on_unused: options.fail_on_unused,
        require_unused_definitions: options.require_unused_definitions,
        definition_count,
        candidate_count,
        passed: violations.is_empty(),
        violations,
    }
}
