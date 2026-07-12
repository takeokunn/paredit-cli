use crate::domain::impact_report::ImpactReportPolicyOptions;
use crate::domain::refactor_plan::RefactorPlanSummary;

use super::types::{ImpactReportPolicy, ImpactRiskLevel};

pub fn evaluate_impact_report_policy(
    options: ImpactReportPolicyOptions,
    summary: &RefactorPlanSummary,
    risk_level: ImpactRiskLevel,
) -> ImpactReportPolicy {
    let mut violations = Vec::new();

    if let Some(threshold) = options.fail_on_risk_level() {
        if risk_level >= threshold {
            violations.push(format!(
                "--fail-on-risk-level {} failed with {} risk",
                threshold.label(),
                risk_level.label()
            ));
        }
    }
    if let Some(required) = options.require_definitions() {
        if summary.definition_count < required {
            violations.push(format!(
                "--require-definitions expected at least {required}, found {}",
                summary.definition_count
            ));
        }
    }
    if let Some(required) = options.require_references() {
        if summary.reference_count < required {
            violations.push(format!(
                "--require-references expected at least {required}, found {}",
                summary.reference_count
            ));
        }
    }
    if let Some(required) = options.require_calls() {
        if summary.call_count < required {
            violations.push(format!(
                "--require-calls expected at least {required}, found {}",
                summary.call_count
            ));
        }
    }

    ImpactReportPolicy {
        fail_on_risk_level: options.fail_on_risk_level(),
        require_definitions: options.require_definitions(),
        require_references: options.require_references(),
        require_calls: options.require_calls(),
        definition_count: summary.definition_count,
        reference_count: summary.reference_count,
        call_count: summary.call_count,
        inbound_edge_count: summary.inbound_edge_count,
        non_call_reference_count: summary.non_call_reference_count,
        signature_mismatch_count: summary.signature_mismatch_count,
        risk_level,
        passed: violations.is_empty(),
        violations,
    }
}
