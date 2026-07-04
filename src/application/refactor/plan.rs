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
    RawRefactorRisk, RefactorOperation, RefactorPlanAutomationDecision,
    RefactorPlanAutomationStatus, RefactorPlanAutomationStep, RefactorPlanAutomationStepStatus,
    RefactorPlanDecision, RefactorPlanGate, RefactorPlanPolicy, RefactorPlanPolicyRequest,
    RefactorPlanRequest, RefactorPlanRiskSummary, RefactorPlanStep, RefactorPlanSummary,
    RefactorRiskLevel, RefactorVerificationCheck, RefactorVerificationRequest, VerificationPhase,
};
pub use verification::refactor_verification_checks;

pub fn build_refactor_plan_decision(request: RefactorPlanRequest<'_>) -> RefactorPlanDecision {
    let gates = refactor_plan_gates(request.operation, &request.summary, request.risks);
    let risk_summary = RefactorPlanRiskSummary::from_gates(&gates);
    let steps = refactor_plan_steps(request.operation, request.symbol, request.files, &gates);
    let policy = evaluate_refactor_plan_policy(request.policy, &request.summary, &gates);
    let automation = refactor_plan_automation_decision(&policy, &steps);

    RefactorPlanDecision {
        gates,
        risk_summary,
        steps,
        policy,
        automation,
    }
}

pub fn refactor_plan_automation_decision(
    policy: &RefactorPlanPolicy,
    steps: &[RefactorPlanStep],
) -> RefactorPlanAutomationDecision {
    let apply_action = steps
        .iter()
        .find(|step| step.order == 3)
        .map(|step| step.action)
        .unwrap_or("review-plan");

    if !policy.passed {
        return RefactorPlanAutomationDecision {
            status: RefactorPlanAutomationStatus::PolicyFailed,
            reason: policy
                .violations
                .first()
                .cloned()
                .unwrap_or_else(|| "refactor plan policy failed".to_owned()),
            next_action: "resolve-policy-violations",
            safe_to_automate: false,
            policy_passed: false,
            blocking_gate_count: policy.blocking_gate_count,
        };
    }

    if policy.blocking_gate_count > 0 {
        return RefactorPlanAutomationDecision {
            status: RefactorPlanAutomationStatus::ManualReview,
            reason: format!(
                "{} blocking gate(s) require manual review before automated edits",
                policy.blocking_gate_count
            ),
            next_action: apply_action,
            safe_to_automate: false,
            policy_passed: true,
            blocking_gate_count: policy.blocking_gate_count,
        };
    }

    RefactorPlanAutomationDecision {
        status: RefactorPlanAutomationStatus::Ready,
        reason: "policy passed and no blocking gates were found".to_owned(),
        next_action: apply_action,
        safe_to_automate: true,
        policy_passed: true,
        blocking_gate_count: 0,
    }
}
