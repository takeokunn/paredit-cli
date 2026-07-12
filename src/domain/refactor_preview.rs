#[derive(Debug, Clone)]
pub struct RefactorPreviewSummary {
    pub file_count: usize,
    pub changed_file_count: usize,
    pub changed_files: Vec<String>,
    pub unchanged_file_count: usize,
    pub written_file_count: usize,
    pub definition_count: usize,
    pub target_occurrence_count: usize,
    pub edit_count: usize,
    pub parse_error_count: usize,
    pub all_outputs_parse: bool,
}

#[derive(Debug, Clone)]
pub struct RefactorPreviewPolicy {
    pub fail_on_no_change: bool,
    pub fail_on_parse_error: bool,
    pub fail_on_target_conflict: bool,
    pub require_changed_files: Option<usize>,
    pub require_definitions: Option<usize>,
    pub require_edits: Option<usize>,
    pub passed: bool,
    pub violations: Vec<String>,
}

impl RefactorPreviewPolicy {
    pub fn summary(&self) -> RefactorPreviewPolicySummary {
        let violation_count = self.violations.len();

        RefactorPreviewPolicySummary {
            violation_count,
            write_blocked: violation_count > 0,
            next_action: if violation_count == 0 {
                "review-preview-or-rerun-with-write"
            } else {
                "review-policy-violations"
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorPreviewPolicySummary {
    pub violation_count: usize,
    pub write_blocked: bool,
    pub next_action: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactorPreviewPolicyOptions {
    fail_on_no_change: bool,
    fail_on_parse_error: bool,
    fail_on_target_conflict: bool,
    require_changed_files: Option<usize>,
    require_definitions: Option<usize>,
    require_edits: Option<usize>,
}

impl RefactorPreviewPolicyOptions {
    pub fn new(
        fail_on_no_change: bool,
        fail_on_parse_error: bool,
        fail_on_target_conflict: bool,
        require_changed_files: Option<usize>,
        require_definitions: Option<usize>,
        require_edits: Option<usize>,
    ) -> Result<Self, String> {
        for (name, value) in [
            ("--require-changed-files", require_changed_files),
            ("--require-definitions", require_definitions),
            ("--require-edits", require_edits),
        ] {
            if value == Some(0) {
                return Err(format!("{name} must be greater than zero when specified"));
            }
        }

        Ok(Self {
            fail_on_no_change,
            fail_on_parse_error,
            fail_on_target_conflict,
            require_changed_files,
            require_definitions,
            require_edits,
        })
    }

    pub const fn fail_on_no_change(self) -> bool {
        self.fail_on_no_change
    }

    pub const fn fail_on_parse_error(self) -> bool {
        self.fail_on_parse_error
    }

    pub const fn fail_on_target_conflict(self) -> bool {
        self.fail_on_target_conflict
    }

    pub const fn require_changed_files(self) -> Option<usize> {
        self.require_changed_files
    }

    pub const fn require_definitions(self) -> Option<usize> {
        self.require_definitions
    }

    pub const fn require_edits(self) -> Option<usize> {
        self.require_edits
    }
}

pub fn evaluate_refactor_preview_policy(
    options: RefactorPreviewPolicyOptions,
    summary: &RefactorPreviewSummary,
) -> RefactorPreviewPolicy {
    let mut violations = Vec::new();

    if options.fail_on_no_change && summary.changed_file_count == 0 {
        violations.push("--fail-on-no-change expected at least one changed file".to_owned());
    }

    if options.fail_on_parse_error && !summary.all_outputs_parse {
        violations.push(format!(
            "--fail-on-parse-error found {} unparsable output file(s)",
            summary.parse_error_count
        ));
    }

    if options.fail_on_target_conflict && summary.target_occurrence_count > 0 {
        violations.push(format!(
            "--fail-on-target-conflict found {} existing replacement symbol occurrence(s)",
            summary.target_occurrence_count
        ));
    }

    if let Some(required) = options.require_changed_files {
        if summary.changed_file_count < required {
            violations.push(format!(
                "--require-changed-files expected at least {required}, found {}",
                summary.changed_file_count
            ));
        }
    }

    if let Some(required) = options.require_definitions {
        if summary.definition_count != required {
            violations.push(format!(
                "--require-definitions expected exactly {required}, found {}",
                summary.definition_count
            ));
        }
    }

    if let Some(required) = options.require_edits {
        if summary.edit_count < required {
            violations.push(format!(
                "--require-edits expected at least {required}, found {}",
                summary.edit_count
            ));
        }
    }

    RefactorPreviewPolicy {
        fail_on_no_change: options.fail_on_no_change,
        fail_on_parse_error: options.fail_on_parse_error,
        fail_on_target_conflict: options.fail_on_target_conflict,
        require_changed_files: options.require_changed_files,
        require_definitions: options.require_definitions,
        require_edits: options.require_edits,
        passed: violations.is_empty(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::RefactorPreviewPolicyOptions;

    #[test]
    fn policy_options_reject_zero_thresholds() {
        let result = RefactorPreviewPolicyOptions::new(true, true, true, Some(0), None, None);

        assert_eq!(
            result.unwrap_err(),
            "--require-changed-files must be greater than zero when specified"
        );
    }
}
