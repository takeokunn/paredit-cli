#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignatureCallStatus {
    Exact,
    MissingArguments,
    ExtraArguments,
    UnknownDefinition,
    AmbiguousDefinition,
}

impl SignatureCallStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::MissingArguments => "missing-arguments",
            Self::ExtraArguments => "extra-arguments",
            Self::UnknownDefinition => "unknown-definition",
            Self::AmbiguousDefinition => "ambiguous-definition",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureReportPolicy {
    pub fail_on_mismatch: bool,
    pub require_definitions: Option<usize>,
    pub require_calls: Option<usize>,
    pub definition_count: usize,
    pub call_count: usize,
    pub mismatch_count: usize,
    pub passed: bool,
    pub violations: Vec<String>,
}

pub fn evaluate_signature_report_policy(
    definition_count: usize,
    statuses: &[SignatureCallStatus],
    fail_on_mismatch: bool,
    require_definitions: Option<usize>,
    require_calls: Option<usize>,
) -> SignatureReportPolicy {
    let call_count = statuses.len();
    let mismatch_count = statuses
        .iter()
        .filter(|status| {
            matches!(
                status,
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
    if let Some(required) = require_definitions {
        if definition_count < required {
            violations.push(format!(
                "--require-definitions expected at least {required}, found {definition_count}"
            ));
        }
    }
    if let Some(required) = require_calls {
        if call_count < required {
            violations.push(format!(
                "--require-calls expected at least {required}, found {call_count}"
            ));
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_mismatch_and_thresholds() {
        let statuses = [
            SignatureCallStatus::MissingArguments,
            SignatureCallStatus::Exact,
        ];
        let policy = evaluate_signature_report_policy(1, &statuses, true, Some(2), Some(3));

        assert_eq!(policy.mismatch_count, 1);
        assert_eq!(policy.violations.len(), 3);
        assert!(!policy.passed);
    }
}
