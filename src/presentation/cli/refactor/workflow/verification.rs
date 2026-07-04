use super::super::super::*;
use super::super::args::*;
use super::super::render::print_refactor_verification;
use super::super::types::verification::RefactorVerification;

pub(in crate::presentation::cli) fn verify_refactor(args: VerifyRefactorArgs) -> Result<()> {
    let operation = ApplicationRefactorOperation::from(args.operation);
    let phase = ApplicationVerificationPhase::from(args.phase);
    let before_files =
        impact_report::collect_impact_reports(&args.files, args.dialect, &args.symbol)?;
    let mut before = summarize_impact_reports(&before_files);
    let gates = application_refactor_plan_gates(operation, &before, raw_refactor_risks(&before));
    before.safe_to_automate = !gates.iter().any(|gate| gate.blocks_automation);

    let after = match (&args.new_symbol, args.phase) {
        (Some(new_symbol), VerificationPhase::Post) => {
            let after_files =
                impact_report::collect_impact_reports(&args.files, args.dialect, new_symbol)?;
            let mut after = summarize_impact_reports(&after_files);
            let after_gates =
                application_refactor_plan_gates(operation, &after, raw_refactor_risks(&after));
            after.safe_to_automate = !after_gates.iter().any(|gate| gate.blocks_automation);
            Some(after)
        }
        _ => None,
    };
    let checks = application_refactor_verification_checks(
        RefactorVerificationRequest {
            operation,
            phase,
            symbol: args.symbol.as_str(),
            new_symbol: args.new_symbol.as_ref().map(|symbol| symbol.as_str()),
            before,
            after,
        },
        &gates,
    );
    let passed = checks.iter().all(|check| check.passed);
    let verification = RefactorVerification {
        operation,
        phase,
        symbol: args.symbol.as_str().to_owned(),
        new_symbol: args.new_symbol.map(|symbol| symbol.as_str().to_owned()),
        passed,
        checks,
        before,
        after,
    };

    print_refactor_verification(&verification, args.output)
}
