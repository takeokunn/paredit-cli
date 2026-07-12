use super::types::{
    RefactorOperation, RefactorPlanGate, RefactorRiskLevel, RefactorVerificationCheck,
    RefactorVerificationRequest, VerificationPhase,
};

pub fn refactor_verification_checks(
    request: RefactorVerificationRequest<'_>,
    gates: &[RefactorPlanGate],
) -> Vec<RefactorVerificationCheck> {
    let mut checks = Vec::new();

    match request.phase {
        VerificationPhase::Pre => {
            checks.push(RefactorVerificationCheck {
                code: "preflight-gates",
                level: RefactorRiskLevel::Error,
                passed: request.before.safe_to_automate,
                message: if request.before.safe_to_automate {
                    "No blocking refactor gates were found.".to_owned()
                } else {
                    "Blocking refactor gates were found; inspect gates before automated editing."
                        .to_owned()
                },
                count: gates.iter().filter(|gate| gate.blocks_automation).count(),
            });

            for gate in gates {
                checks.push(RefactorVerificationCheck {
                    code: gate.code,
                    level: gate.level,
                    passed: !gate.blocks_automation,
                    message: gate.message.clone(),
                    count: gate.count,
                });
            }
        }
        VerificationPhase::Post => {
            if matches!(
                request.operation,
                RefactorOperation::Rename | RefactorOperation::Remove
            ) {
                checks.push(RefactorVerificationCheck {
                    code: "old-symbol-removed",
                    level: RefactorRiskLevel::Error,
                    passed: request.before.reference_count == 0
                        && request.before.definition_count == 0,
                    message: format!(
                        "Old symbol `{}` must have no remaining definitions or references after the refactor.",
                        request.symbol
                    ),
                    count: request.before.reference_count + request.before.definition_count,
                });
            }

            if request.operation == RefactorOperation::Move {
                checks.push(RefactorVerificationCheck {
                    code: "moved-symbol-present",
                    level: RefactorRiskLevel::Error,
                    passed: request
                        .after
                        .map(|after| after.definition_count > 0)
                        .unwrap_or(false),
                    message: format!(
                        "Moved symbol `{}` must still have a discovered definition after the move.",
                        request.symbol
                    ),
                    count: request
                        .after
                        .map(|after| after.definition_count)
                        .unwrap_or(0),
                });
            }

            if request.operation == RefactorOperation::Rename {
                match (request.new_symbol, request.after) {
                    (Some(new_symbol), Some(after)) => {
                        let reference_only_rename_context = after.has_reference_only_rename_context();
                        checks.push(RefactorVerificationCheck {
                            code: "new-symbol-present",
                            level: RefactorRiskLevel::Error,
                            passed: after.reference_count > 0
                                && (after.definition_count > 0 || reference_only_rename_context),
                            message: format!(
                                "New symbol `{new_symbol}` must have at least one reference; a definition is required unless this is a reference-only rename."
                            ),
                            count: after.reference_count + after.definition_count,
                        });
                        if !reference_only_rename_context
                            && !request.target_kind.skips_signature_compatibility()
                        {
                            checks.push(RefactorVerificationCheck {
                                code: "new-symbol-signature-compatible",
                                level: RefactorRiskLevel::Error,
                                passed: after.signature_mismatch_count == 0,
                                message: "New symbol call sites must match discovered callable definitions."
                                    .to_owned(),
                                count: after.signature_mismatch_count,
                            });
                        }
                    }
                    _ => checks.push(RefactorVerificationCheck {
                        code: "new-symbol-required",
                        level: RefactorRiskLevel::Error,
                        passed: false,
                        message:
                            "Post-rename verification requires --new-symbol to inspect replacement impact."
                                .to_owned(),
                        count: 1,
                    }),
                }
            }

            if request.operation == RefactorOperation::Signature
                && !request.target_kind.skips_signature_compatibility()
            {
                checks.push(RefactorVerificationCheck {
                    code: "signature-compatible",
                    level: RefactorRiskLevel::Error,
                    passed: request.before.signature_mismatch_count == 0,
                    message:
                        "Signature refactor must leave every discovered call arity-compatible."
                            .to_owned(),
                    count: request.before.signature_mismatch_count,
                });
            }
        }
    }

    checks
}
