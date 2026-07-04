use crate::application::signature_report::types::{
    SignatureCallStatus, SignatureReportFile, SignatureReportPolicy,
};

pub fn evaluate_signature_report_policy(
    reports: &[SignatureReportFile],
    fail_on_mismatch: bool,
    require_definitions: Option<usize>,
    require_calls: Option<usize>,
) -> SignatureReportPolicy {
    let definition_count = reports
        .iter()
        .map(|report| report.definitions.len())
        .sum::<usize>();
    let call_count = reports
        .iter()
        .map(|report| report.calls.len())
        .sum::<usize>();
    let mismatch_count = reports
        .iter()
        .flat_map(|report| &report.calls)
        .filter(|item| {
            matches!(
                item.status,
                SignatureCallStatus::MissingArguments | SignatureCallStatus::ExtraArguments
            )
        })
        .count();
    let mut violations = Vec::new();

    if fail_on_mismatch && mismatch_count > 0 {
        violations.push(format!(
            "--fail-on-mismatch found {mismatch_count} incompatible call(s)"
        ));
    }
    if let Some(required) = require_definitions
        && definition_count < required
    {
        violations.push(format!(
            "--require-definitions expected at least {required}, found {definition_count}"
        ));
    }
    if let Some(required) = require_calls
        && call_count < required
    {
        violations.push(format!(
            "--require-calls expected at least {required}, found {call_count}"
        ));
    }

    SignatureReportPolicy {
        fail_on_mismatch,
        require_definitions,
        require_calls,
        definition_count,
        call_count,
        mismatch_count,
        passed: violations.is_empty(),
        violations,
    }
}
