use crate::application::usecase::signature_report::types::SignatureReportFile;
use crate::domain::signature_report::{
    SignatureCallStatus, SignatureReportPolicy,
    evaluate_signature_report_policy as evaluate_domain_policy,
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
    let statuses = reports
        .iter()
        .flat_map(|report| &report.calls)
        .map(|item| item.status)
        .collect::<Vec<SignatureCallStatus>>();
    evaluate_domain_policy(
        definition_count,
        &statuses,
        fail_on_mismatch,
        require_definitions,
        require_calls,
    )
}
