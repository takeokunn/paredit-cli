use super::*;
use crate::domain::sexpr::{ByteOffset, ByteSpan};
use proptest::prelude::*;

fn summary() -> RefactorPreviewSummary {
    RefactorPreviewSummary {
        file_count: 2,
        changed_file_count: 1,
        changed_files: vec!["changed.lisp".to_string()],
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
    assert!(
        policy
            .violations
            .iter()
            .any(|violation| violation == "--fail-on-no-change expected at least one changed file")
    );
    assert!(policy.violations.iter().any(|violation| {
        violation == "--fail-on-parse-error found 1 unparsable output file(s)"
    }));
    assert!(policy.violations.iter().any(|violation| {
        violation == "--fail-on-target-conflict found 2 existing replacement symbol occurrence(s)"
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
            changed_files: (0..changed)
                .map(|index| format!("changed-{index}.lisp"))
                .collect(),
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
        let expected_passed = !(fail_on_no_change && changed == 0
            || fail_on_parse_error && !all_outputs_parse
            || fail_on_target_conflict && target_occurrences > 0)
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
