use super::types::{RefactorExecuteDecision, RefactorExecuteGateInputs};

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
