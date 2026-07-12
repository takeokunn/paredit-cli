use std::str::FromStr;

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
}
