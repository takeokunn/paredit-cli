use super::types::{RefactorWriteCandidate, RefactorWritePlan, RefactorWriteRefusal};

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
