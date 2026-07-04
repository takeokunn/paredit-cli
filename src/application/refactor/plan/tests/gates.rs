use super::fixtures::summary;
use super::*;

#[test]
fn rename_plan_blocks_ambiguous_definitions() {
    let gates = refactor_plan_gates(
        RefactorOperation::Rename,
        &summary(),
        vec![RawRefactorRisk {
            level: RefactorRiskLevel::Warning,
            code: "ambiguous-definition",
            message: "multiple definitions".to_owned(),
            count: 2,
        }],
    );

    assert!(
        gates
            .iter()
            .any(|gate| gate.code == "ambiguous-definition" && gate.blocks_automation)
    );
    let policy = evaluate_refactor_plan_policy(
        RefactorPlanPolicyRequest {
            fail_on_blocking_gate: true,
            require_definitions: None,
            require_references: None,
        },
        &summary(),
        &gates,
    );
    assert!(!policy.passed);
}
