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

#[derive(Debug)]
pub struct RefactorPreviewEdit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

#[derive(Debug)]
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

#[derive(Debug, Clone, Copy)]
pub struct RefactorPreviewPolicyOptions {
    pub fail_on_no_change: bool,
    pub fail_on_parse_error: bool,
    pub fail_on_target_conflict: bool,
    pub require_changed_files: Option<usize>,
    pub require_definitions: Option<usize>,
    pub require_edits: Option<usize>,
}
