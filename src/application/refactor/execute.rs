#[derive(Debug, Clone, Copy)]
pub struct RefactorWriteCandidate {
    pub changed: bool,
    pub output_parse_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefactorWriteRefusal {
    UnparsableOutputs { count: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactorWritePlan {
    pub write_requested: bool,
    pub writable_indexes: Vec<usize>,
    pub refusal: Option<RefactorWriteRefusal>,
}

impl RefactorWritePlan {
    pub fn write_allowed(&self) -> bool {
        self.write_requested && self.refusal.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorExecuteGateInputs {
    pub write_requested: bool,
    pub policy_passed: bool,
    pub outputs_parse: bool,
    pub preflight_passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorExecuteDecision {
    pub write_parse_refused: bool,
    pub run_pre_verification: bool,
    pub apply_preview: bool,
    pub run_post_verification: bool,
}

pub fn build_refactor_execute_decision(
    inputs: RefactorExecuteGateInputs,
) -> RefactorExecuteDecision {
    let write_parse_refused = inputs.write_requested && !inputs.outputs_parse;
    let run_pre_verification = inputs.policy_passed && !write_parse_refused;
    let apply_preview = run_pre_verification && inputs.preflight_passed;
    let run_post_verification = inputs.write_requested && apply_preview;

    RefactorExecuteDecision {
        write_parse_refused,
        run_pre_verification,
        apply_preview,
        run_post_verification,
    }
}

pub fn build_refactor_write_plan(
    write_requested: bool,
    candidates: &[RefactorWriteCandidate],
) -> RefactorWritePlan {
    let parse_error_count = candidates
        .iter()
        .filter(|candidate| !candidate.output_parse_ok)
        .count();
    let refusal = (write_requested && parse_error_count > 0).then_some(
        RefactorWriteRefusal::UnparsableOutputs {
            count: parse_error_count,
        },
    );
    let writable_indexes = if write_requested && refusal.is_none() {
        candidates
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| candidate.changed.then_some(index))
            .collect()
    } else {
        Vec::new()
    };

    RefactorWritePlan {
        write_requested,
        writable_indexes,
        refusal,
    }
}

#[cfg(test)]
mod tests {
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
}
