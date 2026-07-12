use std::str::FromStr;

use crate::domain::refactor_plan::RefactorPlanSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactRiskLevel {
    Info,
    Warning,
    Error,
}

impl ImpactRiskLevel {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl FromStr for ImpactRiskLevel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "info" => Ok(Self::Info),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            _ => Err(format!("unknown impact risk level: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImpactReportPolicyOptions {
    fail_on_risk_level: Option<ImpactRiskLevel>,
    require_definitions: Option<usize>,
    require_references: Option<usize>,
    require_calls: Option<usize>,
}

impl ImpactReportPolicyOptions {
    pub fn new(
        fail_on_risk_level: Option<ImpactRiskLevel>,
        require_definitions: Option<usize>,
        require_references: Option<usize>,
        require_calls: Option<usize>,
    ) -> Result<Self, String> {
        Self::validate_threshold("require-definitions", require_definitions)?;
        Self::validate_threshold("require-references", require_references)?;
        Self::validate_threshold("require-calls", require_calls)?;

        Ok(Self {
            fail_on_risk_level,
            require_definitions,
            require_references,
            require_calls,
        })
    }

    fn validate_threshold(label: &str, value: Option<usize>) -> Result<(), String> {
        if matches!(value, Some(0)) {
            return Err(format!("{label} must be greater than zero"));
        }
        Ok(())
    }

    pub const fn fail_on_risk_level(self) -> Option<ImpactRiskLevel> {
        self.fail_on_risk_level
    }

    pub const fn require_definitions(self) -> Option<usize> {
        self.require_definitions
    }

    pub const fn require_references(self) -> Option<usize> {
        self.require_references
    }

    pub const fn require_calls(self) -> Option<usize> {
        self.require_calls
    }
}

#[derive(Debug)]
pub struct ImpactRisk {
    pub level: ImpactRiskLevel,
    pub code: &'static str,
    pub message: String,
    pub count: usize,
}

#[derive(Debug)]
pub struct ImpactReportPolicy {
    pub fail_on_risk_level: Option<ImpactRiskLevel>,
    pub require_definitions: Option<usize>,
    pub require_references: Option<usize>,
    pub require_calls: Option<usize>,
    pub definition_count: usize,
    pub reference_count: usize,
    pub call_count: usize,
    pub inbound_edge_count: usize,
    pub non_call_reference_count: usize,
    pub signature_mismatch_count: usize,
    pub risk_level: ImpactRiskLevel,
    pub passed: bool,
    pub violations: Vec<String>,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_stable() {
        assert_eq!(ImpactRiskLevel::Info.label(), "info");
        assert_eq!(ImpactRiskLevel::Warning.label(), "warning");
        assert_eq!(ImpactRiskLevel::Error.label(), "error");
    }

    #[test]
    fn validates_thresholds() {
        assert!(ImpactReportPolicyOptions::new(None, Some(1), Some(2), Some(3)).is_ok());
        assert_eq!(
            ImpactReportPolicyOptions::new(None, Some(0), None, None).unwrap_err(),
            "require-definitions must be greater than zero"
        );
        assert_eq!(
            ImpactReportPolicyOptions::new(None, None, Some(0), None).unwrap_err(),
            "require-references must be greater than zero"
        );
        assert_eq!(
            ImpactReportPolicyOptions::new(None, None, None, Some(0)).unwrap_err(),
            "require-calls must be greater than zero"
        );
    }

    #[test]
    fn evaluates_policy_failures() {
        let summary = RefactorPlanSummary {
            file_count: 1,
            definition_count: 0,
            reference_count: 1,
            call_count: 0,
            inbound_edge_count: 0,
            outbound_edge_count: 0,
            non_call_reference_count: 1,
            signature_mismatch_count: 0,
            safe_to_automate: false,
        };

        let policy = evaluate_impact_report_policy(
            ImpactReportPolicyOptions::new(
                Some(ImpactRiskLevel::Warning),
                Some(1),
                Some(2),
                Some(1),
            )
            .unwrap(),
            &summary,
            ImpactRiskLevel::Error,
        );

        assert!(!policy.passed);
        assert_eq!(policy.violations.len(), 4);
    }
}
