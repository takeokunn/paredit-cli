use super::*;
use proptest::prelude::*;

#[test]
fn write_plan_refuses_all_writes_when_any_output_does_not_parse() {
    let plan = build_refactor_write_plan(
        true,
        &[
            RefactorWriteCandidate {
                changed: true,
                output_parse_ok: true,
            },
            RefactorWriteCandidate {
                changed: true,
                output_parse_ok: false,
            },
        ],
    );

    assert!(!plan.write_allowed());
    assert_eq!(plan.writable_indexes, Vec::<usize>::new());
    assert_eq!(
        plan.refusal,
        Some(RefactorWriteRefusal::UnparsableOutputs { count: 1 })
    );
}

#[test]
fn execute_decision_refuses_write_parse_failures_before_preflight() {
    let decision = build_refactor_execute_decision(RefactorExecuteGateInputs {
        write_requested: true,
        policy_passed: true,
        outputs_parse: false,
        preflight_passed: true,
    });

    assert!(decision.write_parse_refused);
    assert!(!decision.run_pre_verification);
    assert!(!decision.apply_preview);
    assert!(!decision.run_post_verification);
}

#[test]
fn execute_decision_allows_dry_run_preflight_without_post_verification() {
    let decision = build_refactor_execute_decision(RefactorExecuteGateInputs {
        write_requested: false,
        policy_passed: true,
        outputs_parse: false,
        preflight_passed: true,
    });

    assert!(!decision.write_parse_refused);
    assert!(decision.run_pre_verification);
    assert!(decision.apply_preview);
    assert!(!decision.run_post_verification);
}

proptest! {
    #[test]
    fn pbt_write_plan_only_allows_changed_parseable_outputs_after_write_request(
        write_requested in any::<bool>(),
        candidates in proptest::collection::vec((any::<bool>(), any::<bool>()), 0..32),
    ) {
        let candidates = candidates
            .into_iter()
            .map(|(changed, output_parse_ok)| RefactorWriteCandidate {
                changed,
                output_parse_ok,
            })
            .collect::<Vec<_>>();
        let plan = build_refactor_write_plan(write_requested, &candidates);
        let parse_error_count = candidates
            .iter()
            .filter(|candidate| !candidate.output_parse_ok)
            .count();

        if !write_requested {
            prop_assert!(!plan.write_allowed());
            prop_assert!(plan.writable_indexes.is_empty());
            prop_assert_eq!(plan.refusal, None);
        } else if parse_error_count > 0 {
            prop_assert!(!plan.write_allowed());
            prop_assert!(plan.writable_indexes.is_empty());
            prop_assert_eq!(
                plan.refusal,
                Some(RefactorWriteRefusal::UnparsableOutputs {
                    count: parse_error_count,
                })
            );
        } else {
            let expected_indexes = candidates
                .iter()
                .enumerate()
                .filter_map(|(index, candidate)| candidate.changed.then_some(index))
                .collect::<Vec<_>>();

            prop_assert!(plan.write_allowed());
            prop_assert_eq!(plan.writable_indexes, expected_indexes);
            prop_assert_eq!(plan.refusal, None);
        }
    }

    #[test]
    fn pbt_execute_decision_preserves_gate_order(
        write_requested in any::<bool>(),
        policy_passed in any::<bool>(),
        outputs_parse in any::<bool>(),
        preflight_passed in any::<bool>(),
    ) {
        let inputs = RefactorExecuteGateInputs {
            write_requested,
            policy_passed,
            outputs_parse,
            preflight_passed,
        };
        let decision = build_refactor_execute_decision(inputs);
        let write_parse_refused = write_requested && !outputs_parse;

        prop_assert_eq!(decision.write_parse_refused, write_parse_refused);
        prop_assert_eq!(
            decision.run_pre_verification,
            policy_passed && !write_parse_refused,
        );
        prop_assert_eq!(
            decision.apply_preview,
            policy_passed && !write_parse_refused && preflight_passed,
        );
        prop_assert_eq!(
            decision.run_post_verification,
            write_requested && policy_passed && !write_parse_refused && preflight_passed,
        );

        prop_assert!(!decision.run_post_verification || decision.apply_preview);
        prop_assert!(!decision.apply_preview || decision.run_pre_verification);
        prop_assert!(!decision.run_pre_verification || !decision.write_parse_refused);
    }
}
