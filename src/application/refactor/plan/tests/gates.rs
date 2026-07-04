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

#[test]
fn risk_summary_counts_gate_occurrences_by_level_and_blocking_status() {
    let gates = vec![
        RefactorPlanGate {
            level: RefactorRiskLevel::Info,
            code: "context",
            message: "context only".to_owned(),
            count: 2,
            blocks_automation: false,
        },
        RefactorPlanGate {
            level: RefactorRiskLevel::Warning,
            code: "manual-review",
            message: "needs review".to_owned(),
            count: 3,
            blocks_automation: true,
        },
        RefactorPlanGate {
            level: RefactorRiskLevel::Error,
            code: "parse-error",
            message: "cannot parse".to_owned(),
            count: 1,
            blocks_automation: true,
        },
    ];

    let summary = RefactorPlanRiskSummary::from_gates(&gates);

    assert_eq!(summary.highest_level, Some(RefactorRiskLevel::Error));
    assert_eq!(summary.info_count, 2);
    assert_eq!(summary.warning_count, 3);
    assert_eq!(summary.error_count, 1);
    assert_eq!(summary.blocking_count, 4);
    assert_eq!(summary.advisory_count, 2);
}
