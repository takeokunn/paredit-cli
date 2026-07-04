use super::types::{RefactorPreviewPolicy, RefactorPreviewPolicyOptions, RefactorPreviewSummary};

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
