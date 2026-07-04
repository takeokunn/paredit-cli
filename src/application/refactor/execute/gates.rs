use super::types::{
    RefactorExecuteDecision, RefactorExecuteDecisionStatus, RefactorExecuteGateInputs,
};

pub fn build_refactor_execute_decision(
    inputs: RefactorExecuteGateInputs,
) -> RefactorExecuteDecision {
    let write_parse_refused = inputs.write_requested && !inputs.outputs_parse;
    let run_pre_verification = inputs.policy_passed && !write_parse_refused;
    let apply_preview = run_pre_verification && inputs.preflight_passed;
    let run_post_verification = inputs.write_requested && apply_preview;
    let status = if !inputs.policy_passed {
        RefactorExecuteDecisionStatus::BlockedByPolicy
    } else if write_parse_refused {
        RefactorExecuteDecisionStatus::RefusedUnparsableOutput
    } else if !inputs.preflight_passed {
        RefactorExecuteDecisionStatus::BlockedByPreVerification
    } else if inputs.write_requested {
        RefactorExecuteDecisionStatus::ReadyToWrite
    } else {
        RefactorExecuteDecisionStatus::DryRunReady
    };

    RefactorExecuteDecision {
        status,
        write_parse_refused,
        run_pre_verification,
        apply_preview,
        run_post_verification,
    }
}
