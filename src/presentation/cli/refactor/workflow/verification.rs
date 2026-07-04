use super::super::super::*;
use super::super::args::*;
use super::super::render::print_refactor_verification;
use super::super::types::verification::RefactorVerification;

pub(in crate::presentation::cli) fn verify_refactor(args: VerifyRefactorArgs) -> Result<()> {
    let verification = build_refactor_verification(
        &args.files,
        args.dialect,
        &args.symbol,
        args.new_symbol.as_ref(),
        args.operation,
        args.phase,
    )?;

    print_refactor_verification(&verification, args.output)
}

pub(super) fn build_refactor_verification(
    paths: &[PathBuf],
    dialect: Option<DialectArg>,
    symbol: &SymbolName,
    new_symbol: Option<&SymbolName>,
    operation: RefactorOperation,
    phase: VerificationPhase,
) -> Result<RefactorVerification> {
    let application_operation = ApplicationRefactorOperation::from(operation);
    let application_phase = ApplicationVerificationPhase::from(phase);
    let before_files = impact_report::workflow::collect_impact_reports(paths, dialect, symbol)?;
    let mut before = summarize_impact_reports(&before_files);
    let gates = application_refactor_plan_gates(
        application_operation,
        &before,
        raw_refactor_risks(&before),
    );
    before.safe_to_automate = !gates.iter().any(|gate| gate.blocks_automation);

    let after = match (new_symbol, phase) {
        (Some(new_symbol), VerificationPhase::Post) => {
            let after_files =
                impact_report::workflow::collect_impact_reports(paths, dialect, new_symbol)?;
            let mut after = summarize_impact_reports(&after_files);
            let after_gates = application_refactor_plan_gates(
                application_operation,
                &after,
                raw_refactor_risks(&after),
            );
            after.safe_to_automate = !after_gates.iter().any(|gate| gate.blocks_automation);
            Some(after)
        }
        _ => None,
    };
    let checks = application_refactor_verification_checks(
        RefactorVerificationRequest {
            operation: application_operation,
            phase: application_phase,
            symbol: symbol.as_str(),
            new_symbol: new_symbol.map(|symbol| symbol.as_str()),
            before,
            after,
        },
        &gates,
    );
    let passed = checks.iter().all(|check| check.passed);
    Ok(RefactorVerification {
        operation: application_operation,
        phase: application_phase,
        symbol: symbol.as_str().to_owned(),
        new_symbol: new_symbol.map(|symbol| symbol.as_str().to_owned()),
        passed,
        checks,
        before,
        after,
    })
}
