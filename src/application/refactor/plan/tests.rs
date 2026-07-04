use super::*;

fn summary() -> RefactorPlanSummary {
    RefactorPlanSummary {
        file_count: 2,
        definition_count: 1,
        reference_count: 3,
        call_count: 2,
        inbound_edge_count: 0,
        outbound_edge_count: 0,
        non_call_reference_count: 0,
        signature_mismatch_count: 0,
        safe_to_automate: true,
    }
}

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
fn post_rename_verification_requires_old_symbol_removed_and_new_symbol_present() {
    let before = RefactorPlanSummary {
        definition_count: 0,
        reference_count: 0,
        ..summary()
    };
    let after = RefactorPlanSummary {
        definition_count: 1,
        reference_count: 2,
        signature_mismatch_count: 0,
        ..summary()
    };
    let checks = refactor_verification_checks(
        RefactorVerificationRequest {
            operation: RefactorOperation::Rename,
            phase: VerificationPhase::Post,
            symbol: "old-name",
            new_symbol: Some("new-name"),
            before,
            after: Some(after),
        },
        &[],
    );

    assert!(checks.iter().all(|check| check.passed));
    assert!(
        checks
            .iter()
            .any(|check| check.code == "old-symbol-removed")
    );
    assert!(
        checks
            .iter()
            .any(|check| check.code == "new-symbol-present")
    );
    assert!(
        checks
            .iter()
            .any(|check| check.code == "new-symbol-signature-compatible")
    );
}
