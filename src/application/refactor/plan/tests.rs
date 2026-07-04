use super::*;
use proptest::prelude::*;
use std::path::PathBuf;

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

#[test]
fn remove_plan_uses_unused_definition_cleanup_usecase() {
    let files = vec![PathBuf::from("src/core.lisp"), PathBuf::from("src/ui.el")];
    let steps = refactor_plan_steps(RefactorOperation::Remove, "stale-helper", &files, &[]);

    let apply = steps
        .iter()
        .find(|step| step.order == 3)
        .expect("apply step");
    assert_eq!(apply.action, "apply-unused-definition-removal");
    let apply_command = apply.command.as_deref().expect("apply command");
    assert!(apply_command.contains("paredit remove-unused-definitions --output json"));
    assert!(apply_command.contains("'src/core.lisp'"));
    assert!(apply_command.contains("'src/ui.el'"));

    let verify = steps
        .iter()
        .find(|step| step.order == 4)
        .expect("verify step");
    let verify_command = verify.command.as_deref().expect("verify command");
    assert!(verify_command.contains(
        "paredit verify-refactor --symbol 'stale-helper' --operation remove --phase post --output json"
    ));
    assert!(!verify_command.contains("--require-definitions 1"));
    assert!(!verify_command.contains("--require-references 1"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pre_rename_verification_passes_iff_no_gate_blocks_automation(
        blocking_gate_count in 0usize..4,
        nonblocking_gate_count in 0usize..4,
    ) {
        let mut gates = Vec::new();
        for index in 0..blocking_gate_count {
            gates.push(RefactorPlanGate {
                level: RefactorRiskLevel::Error,
                code: "blocking-risk",
                message: format!("blocking risk {index}"),
                count: index + 1,
                blocks_automation: true,
            });
        }
        for index in 0..nonblocking_gate_count {
            gates.push(RefactorPlanGate {
                level: RefactorRiskLevel::Warning,
                code: "advisory-risk",
                message: format!("advisory risk {index}"),
                count: index + 1,
                blocks_automation: false,
            });
        }
        let before = RefactorPlanSummary {
            safe_to_automate: blocking_gate_count == 0,
            ..summary()
        };

        let checks = refactor_verification_checks(
            RefactorVerificationRequest {
                operation: RefactorOperation::Rename,
                phase: VerificationPhase::Pre,
                symbol: "old-name",
                new_symbol: Some("new-name"),
                before,
                after: None,
            },
            &gates,
        );

        prop_assert_eq!(
            checks.iter().all(|check| check.passed),
            blocking_gate_count == 0
        );
        let preflight = checks
            .iter()
            .find(|check| check.code == "preflight-gates")
            .expect("preflight check");
        prop_assert_eq!(preflight.passed, blocking_gate_count == 0);
        prop_assert_eq!(preflight.count, blocking_gate_count);
    }

    #[test]
    fn post_rename_verification_passes_iff_old_symbol_is_removed_and_new_symbol_is_usable(
        old_definitions in 0usize..4,
        old_references in 0usize..4,
        new_definitions in 0usize..4,
        new_references in 0usize..4,
        signature_mismatches in 0usize..4,
    ) {
        let before = RefactorPlanSummary {
            definition_count: old_definitions,
            reference_count: old_references,
            ..summary()
        };
        let after = RefactorPlanSummary {
            definition_count: new_definitions,
            reference_count: new_references,
            signature_mismatch_count: signature_mismatches,
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
        let expected_passed = old_definitions == 0
            && old_references == 0
            && new_definitions > 0
            && new_references > 0
            && signature_mismatches == 0;

        prop_assert_eq!(
            checks.iter().all(|check| check.passed),
            expected_passed
        );
        prop_assert_eq!(
            checks
                .iter()
                .find(|check| check.code == "old-symbol-removed")
                .expect("old symbol check")
                .passed,
            old_definitions == 0 && old_references == 0
        );
        prop_assert_eq!(
            checks
                .iter()
                .find(|check| check.code == "new-symbol-present")
                .expect("new symbol check")
                .passed,
            new_definitions > 0 && new_references > 0
        );
        prop_assert_eq!(
            checks
                .iter()
                .find(|check| check.code == "new-symbol-signature-compatible")
                .expect("signature check")
                .passed,
            signature_mismatches == 0
        );
    }

    #[test]
    fn remove_plan_apply_step_tracks_blocking_gate_invariants(
        blocking_gate_count in 0usize..4,
        nonblocking_gate_count in 0usize..4,
    ) {
        let mut gates = Vec::new();
        for index in 0..blocking_gate_count {
            gates.push(RefactorPlanGate {
                level: RefactorRiskLevel::Error,
                code: "blocking-risk",
                message: format!("blocking risk {index}"),
                count: index + 1,
                blocks_automation: true,
            });
        }
        for index in 0..nonblocking_gate_count {
            gates.push(RefactorPlanGate {
                level: RefactorRiskLevel::Warning,
                code: "advisory-risk",
                message: format!("advisory risk {index}"),
                count: index + 1,
                blocks_automation: false,
            });
        }

        let files = vec![PathBuf::from("src/core.lisp")];
        let steps = refactor_plan_steps(RefactorOperation::Remove, "stale-helper", &files, &gates);
        let apply = steps
            .iter()
            .find(|step| step.order == 3)
            .expect("apply step");

        if blocking_gate_count == 0 {
            prop_assert_eq!(apply.action, "apply-unused-definition-removal");
            prop_assert!(
                apply
                    .command
                    .as_deref()
                    .is_some_and(|command| command.contains("paredit remove-unused-definitions --output json"))
            );
        } else {
            prop_assert_eq!(apply.action, "review-remove-scope");
            prop_assert!(apply.command.is_none());
        }
    }
}
