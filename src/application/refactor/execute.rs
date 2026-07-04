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
    }
}
