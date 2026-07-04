mod gates;
mod policy;
mod steps;
#[cfg(test)]
mod tests;
mod types;
mod verification;

pub use gates::refactor_plan_gates;
pub use policy::evaluate_refactor_plan_policy;
pub use steps::refactor_plan_steps;
pub use types::{
    RawRefactorRisk, RefactorOperation, RefactorPlanDecision, RefactorPlanGate, RefactorPlanPolicy,
    RefactorPlanPolicyRequest, RefactorPlanRequest, RefactorPlanStep, RefactorPlanSummary,
    RefactorRiskLevel, RefactorVerificationCheck, RefactorVerificationRequest, VerificationPhase,
};
pub use verification::refactor_verification_checks;

pub fn build_refactor_plan_decision(request: RefactorPlanRequest<'_>) -> RefactorPlanDecision {
    let gates = refactor_plan_gates(request.operation, &request.summary, request.risks);
    let steps = refactor_plan_steps(request.operation, request.symbol, request.files, &gates);
    let policy = evaluate_refactor_plan_policy(request.policy, &request.summary, &gates);

    RefactorPlanDecision {
        gates,
        steps,
        policy,
    }
}
