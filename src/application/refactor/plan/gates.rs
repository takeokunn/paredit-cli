use super::types::{
    RawRefactorRisk, RefactorOperation, RefactorPlanGate, RefactorPlanSummary, RefactorRiskLevel,
};

pub fn refactor_plan_gates(
    operation: RefactorOperation,
    summary: &RefactorPlanSummary,
    risks: Vec<RawRefactorRisk>,
) -> Vec<RefactorPlanGate> {
    let mut gates = risks
        .into_iter()
        .map(|risk| {
            let blocks_automation = risk.level == RefactorRiskLevel::Error
                || match (operation, risk.code) {
                    (RefactorOperation::Rename, "signature-mismatch" | "ambiguous-definition") => {
                        true
                    }
                    (
                        RefactorOperation::Remove | RefactorOperation::Move,
                        "inbound-callers" | "ambiguous-definition",
                    ) => true,
                    (
                        RefactorOperation::Signature,
                        "inbound-callers"
                        | "non-call-references"
                        | "signature-mismatch"
                        | "ambiguous-definition",
                    ) => true,
                    _ => false,
                };
            RefactorPlanGate {
                level: risk.level,
                code: risk.code,
                message: risk.message,
                count: risk.count,
                blocks_automation,
            }
        })
        .collect::<Vec<_>>();

    if operation == RefactorOperation::Remove && summary.reference_count > summary.definition_count
    {
        gates.push(RefactorPlanGate {
            level: RefactorRiskLevel::Warning,
            code: "external-references",
            message: "The symbol has references outside its own definition; removal needs caller and reference cleanup."
                .to_owned(),
            count: summary.reference_count.saturating_sub(summary.definition_count),
            blocks_automation: true,
        });
    }

    gates
}
