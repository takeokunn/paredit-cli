use super::fixtures::summary;
use super::*;

#[test]
fn rename_plan_blocks_ambiguous_definitions() {
    let gates = refactor_plan_gates(
        RefactorOperation::Rename,
        RefactorPlanTargetKind::Callable,
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
fn rename_plan_does_not_block_signature_mismatch_for_macro_targets() {
    let gates = refactor_plan_gates(
        RefactorOperation::Rename,
        RefactorPlanTargetKind::Macro,
        &summary(),
        vec![RawRefactorRisk {
            level: RefactorRiskLevel::Warning,
            code: "signature-mismatch",
            message: "macro-like definition".to_owned(),
            count: 1,
        }],
    );

    assert!(
        gates
            .iter()
            .any(|gate| gate.code == "signature-mismatch" && !gate.blocks_automation)
    );
}

#[test]
fn rename_plan_does_not_block_signature_mismatch_for_macro_like_targets() {
    for target_kind in [
        RefactorPlanTargetKind::CompilerMacro,
        RefactorPlanTargetKind::SetfExpander,
        RefactorPlanTargetKind::SymbolMacro,
    ] {
        let gates = refactor_plan_gates(
            RefactorOperation::Rename,
            target_kind,
            &summary(),
            vec![RawRefactorRisk {
                level: RefactorRiskLevel::Warning,
                code: "signature-mismatch",
                message: "macro-like definition".to_owned(),
                count: 1,
            }],
        );

        assert!(
            gates
                .iter()
                .any(|gate| gate.code == "signature-mismatch" && !gate.blocks_automation)
        );
    }
}

#[test]
fn symbol_macro_target_kind_is_treated_as_macro_like_for_signature_compatibility() {
    assert!(RefactorPlanTargetKind::SymbolMacro.is_macro_like());
    assert!(RefactorPlanTargetKind::SymbolMacro.skips_signature_compatibility());
}

#[test]
fn symbol_macro_target_kind_skips_call_coverage_for_non_signature_operations() {
    assert!(!RefactorPlanTargetKind::SymbolMacro.requires_call_coverage(RefactorOperation::Rename));
    assert!(!RefactorPlanTargetKind::SymbolMacro.requires_call_coverage(RefactorOperation::Move));
    assert!(RefactorPlanTargetKind::SymbolMacro.requires_call_coverage(RefactorOperation::Remove));
    assert!(
        !RefactorPlanTargetKind::SymbolMacro.requires_call_coverage(RefactorOperation::Signature)
    );
}

#[test]
fn rename_plan_still_blocks_signature_mismatch_for_callable_targets() {
    let gates = refactor_plan_gates(
        RefactorOperation::Rename,
        RefactorPlanTargetKind::Callable,
        &summary(),
        vec![RawRefactorRisk {
            level: RefactorRiskLevel::Warning,
            code: "signature-mismatch",
            message: "callable signature changed".to_owned(),
            count: 1,
        }],
    );

    assert!(
        gates
            .iter()
            .any(|gate| gate.code == "signature-mismatch" && gate.blocks_automation)
    );
}

#[test]
fn rename_plan_allows_reference_only_context_without_blocking_no_definition_or_signature_mismatch()
{
    let summary = RefactorPlanSummary {
        file_count: 1,
        definition_count: 0,
        reference_count: 2,
        call_count: 1,
        inbound_edge_count: 0,
        outbound_edge_count: 0,
        non_call_reference_count: 0,
        signature_mismatch_count: 1,
        safe_to_automate: false,
    };

    let gates = refactor_plan_gates(
        RefactorOperation::Rename,
        RefactorPlanTargetKind::Callable,
        &summary,
        vec![
            RawRefactorRisk {
                level: RefactorRiskLevel::Error,
                code: "no-definition",
                message: "no definition discovered".to_owned(),
                count: 0,
            },
            RawRefactorRisk {
                level: RefactorRiskLevel::Warning,
                code: "signature-mismatch",
                message: "reference-only rename".to_owned(),
                count: 1,
            },
        ],
    );

    assert!(
        gates
            .iter()
            .all(|gate| gate.code != "no-definition" || !gate.blocks_automation)
    );
    assert!(
        gates
            .iter()
            .all(|gate| gate.code != "signature-mismatch" || !gate.blocks_automation)
    );
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
