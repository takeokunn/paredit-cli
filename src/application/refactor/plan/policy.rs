use super::types::{
    RefactorPlanGate, RefactorPlanPolicy, RefactorPlanPolicyRequest, RefactorPlanSummary,
};

pub fn evaluate_refactor_plan_policy(
    request: RefactorPlanPolicyRequest,
    summary: &RefactorPlanSummary,
    gates: &[RefactorPlanGate],
) -> RefactorPlanPolicy {
    let mut violations = Vec::new();
    let blocking_gate_count = gates.iter().filter(|gate| gate.blocks_automation).count();

    if request.fail_on_blocking_gate && blocking_gate_count > 0 {
        violations.push(format!(
            "--fail-on-blocking-gate found {blocking_gate_count} blocking gate(s)"
        ));
    }

    if let Some(required) = request.require_definitions {
        if summary.definition_count < required {
            violations.push(format!(
                "--require-definitions expected at least {required}, found {}",
                summary.definition_count
            ));
        }
    }

    if let Some(required) = request.require_references {
        if summary.reference_count < required {
            violations.push(format!(
                "--require-references expected at least {required}, found {}",
                summary.reference_count
            ));
        }
    }

    RefactorPlanPolicy {
        fail_on_blocking_gate: request.fail_on_blocking_gate,
        require_definitions: request.require_definitions,
        require_references: request.require_references,
        blocking_gate_count,
        definition_count: summary.definition_count,
        reference_count: summary.reference_count,
        passed: violations.is_empty(),
        violations,
    }
}
