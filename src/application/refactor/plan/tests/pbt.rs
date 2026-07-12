use super::fixtures::{gates, summary};
use super::*;
use proptest::prelude::*;
use std::path::PathBuf;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pre_rename_verification_passes_iff_no_gate_blocks_automation(
        blocking_gate_count in 0usize..4,
        nonblocking_gate_count in 0usize..4,
    ) {
        let gates = gates(blocking_gate_count, nonblocking_gate_count);
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
                target_kind: RefactorPlanTargetKind::Callable,
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
    fn post_rename_verification_allows_reference_only_new_symbol_context(
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
        // A parameter or local binding can be renamed without a discoverable definition.
        let new_symbol_is_reference_only = after.has_reference_only_rename_context();

        let checks = refactor_verification_checks(
            RefactorVerificationRequest {
                operation: RefactorOperation::Rename,
                phase: VerificationPhase::Post,
                symbol: "old-name",
                new_symbol: Some("new-name"),
                target_kind: RefactorPlanTargetKind::Callable,
                before,
                after: Some(after),
            },
            &[],
        );
        let expected_passed = old_definitions == 0
            && old_references == 0
            && new_references > 0
            && (new_symbol_is_reference_only || signature_mismatches == 0);

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
            new_references > 0 && (new_definitions > 0 || new_symbol_is_reference_only)
        );
        let signature_check = checks
            .iter()
            .find(|check| check.code == "new-symbol-signature-compatible");
        if new_symbol_is_reference_only {
            prop_assert!(signature_check.is_none());
        } else {
            prop_assert_eq!(
                signature_check.expect("signature check").passed,
                signature_mismatches == 0
            );
        }
    }

    #[test]
    fn remove_plan_apply_step_tracks_blocking_gate_invariants(
        blocking_gate_count in 0usize..4,
        nonblocking_gate_count in 0usize..4,
    ) {
        let gates = gates(blocking_gate_count, nonblocking_gate_count);
        let files = vec![PathBuf::from("src/core.lisp")];
        let steps = refactor_plan_steps(
            RefactorOperation::Remove,
            "stale-helper",
            &files,
            RefactorPlanTargetKind::Unknown,
            &gates,
        );
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
                    .is_some_and(|command| command.contains("paredit refactor remove-unused-definitions --output json"))
            );
        } else {
            prop_assert_eq!(apply.action, "review-remove-scope");
            prop_assert!(apply.command.is_none());
        }
    }
}
