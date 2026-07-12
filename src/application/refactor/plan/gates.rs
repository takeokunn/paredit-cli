use super::types::{
    RawRefactorRisk, RefactorOperation, RefactorPlanGate, RefactorPlanSummary,
    RefactorPlanTargetKind, RefactorRiskLevel,
};

pub fn refactor_plan_gates(
    operation: RefactorOperation,
    target_kind: RefactorPlanTargetKind,
    summary: &RefactorPlanSummary,
    risks: Vec<RawRefactorRisk>,
) -> Vec<RefactorPlanGate> {
    let reference_only_rename_context =
        operation == RefactorOperation::Rename && summary.has_reference_only_rename_context();
    let mut gates = risks
        .into_iter()
        .map(|risk| {
            // A binding or parameter rename can have references without a discoverable definition.
            let reference_only_rename_advisory = reference_only_rename_context
                && matches!(risk.code, "no-definition" | "signature-mismatch");
            let blocks_automation = !reference_only_rename_advisory
                && (risk.level == RefactorRiskLevel::Error
                    || match operation {
                        RefactorOperation::Rename => match risk.code {
                            "ambiguous-definition" => true,
                            "no-definition" => !reference_only_rename_context,
                            "signature-mismatch" => {
                                !reference_only_rename_context
                                    && !target_kind.skips_signature_compatibility()
                            }
                            _ => false,
                        },
                        RefactorOperation::Remove | RefactorOperation::Move => {
                            matches!(risk.code, "inbound-callers" | "ambiguous-definition")
                        }
                        RefactorOperation::Signature => {
                            matches!(
                                risk.code,
                                "inbound-callers"
                                    | "non-call-references"
                                    | "signature-mismatch"
                                    | "ambiguous-definition"
                            ) && !(risk.code == "signature-mismatch"
                                && target_kind.skips_signature_compatibility())
                        }
                    });
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
