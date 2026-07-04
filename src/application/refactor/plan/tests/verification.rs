use super::fixtures::summary;
use super::*;

#[test]
fn post_rename_verification_requires_old_symbol_removed_and_new_symbol_present() {
    let before = RefactorPlanSummary {
        definition_count: 0,
        reference_count: 0,
        ..summary()
    };
    let after = RefactorPlanSummary {
        definition_count: 1,
        reference_count: 2,
        signature_mismatch_count: 0,
        ..summary()
    };
    let checks = refactor_verification_checks(
        RefactorVerificationRequest {
            operation: RefactorOperation::Rename,
            phase: VerificationPhase::Post,
            symbol: "old-name",
            new_symbol: Some("new-name"),
            before,
            after: Some(after),
        },
        &[],
    );

    assert!(checks.iter().all(|check| check.passed));
    assert!(
        checks
            .iter()
            .any(|check| check.code == "old-symbol-removed")
    );
    assert!(
        checks
            .iter()
            .any(|check| check.code == "new-symbol-present")
    );
    assert!(
        checks
            .iter()
            .any(|check| check.code == "new-symbol-signature-compatible")
    );
}
