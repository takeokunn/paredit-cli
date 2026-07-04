use super::fixtures::summary;
use super::*;

#[test]
fn policy_fails_on_blocking_gates_and_required_counts() {
    let gates = vec![RefactorPlanGate {
        level: RefactorRiskLevel::Error,
        code: "ambiguous-definition",
        message: "ambiguous definition".to_owned(),
        count: 2,
        blocks_automation: true,
    }];
    let policy = evaluate_refactor_plan_policy(
        RefactorPlanPolicyRequest {
            fail_on_blocking_gate: true,
            require_definitions: Some(2),
            require_references: Some(4),
        },
        &summary(),
        &gates,
    );

    assert!(!policy.passed);
    assert_eq!(policy.blocking_gate_count, 1);
    assert_eq!(policy.violations.len(), 3);
    assert!(
        policy
            .violations
            .iter()
            .any(|violation| { violation == "--fail-on-blocking-gate found 1 blocking gate(s)" })
    );
    assert!(
        policy
            .violations
            .iter()
            .any(|violation| { violation == "--require-definitions expected at least 2, found 1" })
    );
    assert!(
        policy
            .violations
            .iter()
            .any(|violation| { violation == "--require-references expected at least 4, found 3" })
    );
}
