use crate::domain::sexpr::ByteSpan;

#[derive(Debug, Clone, Copy)]
pub struct RefactorPreviewSummary {
    pub file_count: usize,
    pub changed_file_count: usize,
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

#[derive(Debug, Clone, Copy)]
pub struct RefactorPreviewPolicyOptions {
    pub fail_on_no_change: bool,
    pub fail_on_parse_error: bool,
    pub fail_on_target_conflict: bool,
    pub require_changed_files: Option<usize>,
    pub require_definitions: Option<usize>,
    pub require_edits: Option<usize>,
}

pub fn refactor_preview_edits(edits: &[(ByteSpan, String)]) -> Vec<RefactorPreviewEdit> {
    let mut preview_edits = edits
        .iter()
        .map(|(span, replacement)| RefactorPreviewEdit {
            start: span.start().get(),
            end: span.end().get(),
            replacement: replacement.clone(),
        })
        .collect::<Vec<_>>();
    preview_edits.sort_by_key(|edit| (edit.start, edit.end));
    preview_edits
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
    use super::*;
    use crate::domain::sexpr::ByteOffset;
    use proptest::prelude::*;

    fn summary() -> RefactorPreviewSummary {
        RefactorPreviewSummary {
            file_count: 2,
            changed_file_count: 1,
            unchanged_file_count: 1,
            written_file_count: 0,
            definition_count: 1,
            target_occurrence_count: 0,
            edit_count: 3,
            parse_error_count: 0,
            all_outputs_parse: true,
        }
    }

    #[test]
    fn policy_reports_every_failed_preview_gate() {
        let summary = RefactorPreviewSummary {
            changed_file_count: 0,
            definition_count: 1,
            target_occurrence_count: 2,
            edit_count: 1,
            parse_error_count: 1,
            all_outputs_parse: false,
            ..summary()
        };
        let policy = evaluate_refactor_preview_policy(
            RefactorPreviewPolicyOptions {
                fail_on_no_change: true,
                fail_on_parse_error: true,
                fail_on_target_conflict: true,
                require_changed_files: Some(2),
                require_definitions: Some(2),
                require_edits: Some(4),
            },
            &summary,
        );

        assert!(!policy.passed);
        assert_eq!(policy.violations.len(), 6);
        assert!(policy.violations.iter().any(|violation| {
            violation == "--fail-on-no-change expected at least one changed file"
        }));
        assert!(policy.violations.iter().any(|violation| {
            violation == "--fail-on-parse-error found 1 unparsable output file(s)"
        }));
        assert!(policy.violations.iter().any(|violation| {
            violation
                == "--fail-on-target-conflict found 2 existing replacement symbol occurrence(s)"
        }));
    }

    proptest! {
        #[test]
        fn pbt_policy_passes_iff_no_preview_gate_fails(
            changed in 0usize..8,
            definitions in 0usize..8,
            target_occurrences in 0usize..8,
            edits in 0usize..16,
            parse_errors in 0usize..8,
            require_changed in proptest::option::of(0usize..8),
            require_definitions in proptest::option::of(0usize..8),
            require_edits in proptest::option::of(0usize..16),
            fail_on_no_change in any::<bool>(),
            fail_on_parse_error in any::<bool>(),
            fail_on_target_conflict in any::<bool>(),
        ) {
            let all_outputs_parse = parse_errors == 0;
            let summary = RefactorPreviewSummary {
                file_count: changed + 1,
                changed_file_count: changed,
                unchanged_file_count: 1,
                written_file_count: 0,
                definition_count: definitions,
                target_occurrence_count: target_occurrences,
                edit_count: edits,
                parse_error_count: parse_errors,
                all_outputs_parse,
            };
            let options = RefactorPreviewPolicyOptions {
                fail_on_no_change,
                fail_on_parse_error,
                fail_on_target_conflict,
                require_changed_files: require_changed,
                require_definitions,
                require_edits,
            };

            let policy = evaluate_refactor_preview_policy(options, &summary);
            let expected_passed = !(fail_on_no_change && changed == 0)
                && !(fail_on_parse_error && !all_outputs_parse)
                && !(fail_on_target_conflict && target_occurrences > 0)
                && require_changed.is_none_or(|required| changed >= required)
                && require_definitions.is_none_or(|required| definitions == required)
                && require_edits.is_none_or(|required| edits >= required);

            prop_assert_eq!(policy.passed, expected_passed);
            prop_assert_eq!(policy.violations.is_empty(), expected_passed);
        }

        #[test]
        fn pbt_preview_edits_are_sorted_by_span_start_then_end(
            spans in proptest::collection::vec((0usize..128, 0usize..128, "[a-z]{1,8}"), 0..32)
        ) {
            let raw_edits = spans
                .into_iter()
                .map(|(a, b, replacement)| {
                    let start = a.min(b);
                    let end = a.max(b);
                    (
                        ByteSpan::new(ByteOffset::new(start), ByteOffset::new(end)),
                        replacement,
                    )
                })
                .collect::<Vec<_>>();

            let preview_edits = refactor_preview_edits(&raw_edits);

            prop_assert!(preview_edits
                .windows(2)
                .all(|pair| (pair[0].start, pair[0].end) <= (pair[1].start, pair[1].end)));
            prop_assert_eq!(preview_edits.len(), raw_edits.len());
        }
    }
}
