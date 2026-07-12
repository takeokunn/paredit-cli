use super::*;

#[test]
fn max_results_truncates_only_reported_pairs() {
    let values = candidates("a.lisp", "(foo a) (foo b) (foo c)", 2);
    let report = build_similarity_pairs(values, 0.0, SimilarityOverlapPolicy::All, Some(1));
    assert_eq!(report.summary.matched_pairs, 3);
    assert_eq!(report.summary.reported_pairs, 1);
    assert!(report.summary.truncated);
    assert_eq!(report.pairs.len(), 1);

    let unlimited = build_similarity_pairs(
        candidates("a.lisp", "(foo a) (foo b) (foo c)", 2),
        0.0,
        SimilarityOverlapPolicy::All,
        None,
    );
    assert_eq!(
        report.summary.matched_pairs,
        unlimited.summary.matched_pairs
    );
    assert_eq!(unlimited.summary.reported_pairs, 3);
    assert!(!unlimited.summary.truncated);
}

#[test]
fn max_comparisons_stops_ted_evaluation_and_tracks_unprocessed_pairs() {
    let report = super::reports::build_similarity_pairs(
        candidates("a.lisp", "(foo a) (foo b) (foo c)", 2),
        &SimilarityReportOptions {
            threshold: 0.0,
            overlap_policy: SimilarityOverlapPolicy::All,
            max_comparisons: Some(1),
            ..SimilarityReportOptions::default()
        },
    );

    assert_eq!(report.summary.possible_pairs, 3);
    assert_eq!(report.summary.evaluated_pairs, 1);
    assert_eq!(report.summary.pruned_by_size, 0);
    assert_eq!(report.summary.unprocessed_pairs, 2);
    assert!(report.summary.comparison_limit_reached);
    assert_eq!(report.summary.matched_pairs, 1);
    assert_eq!(
        report.summary.possible_pairs,
        report.summary.evaluated_pairs
            + report.summary.pruned_by_size
            + report.summary.unprocessed_pairs
    );
}

#[test]
fn sufficient_max_comparisons_does_not_report_limit_reached() {
    let report = super::reports::build_similarity_pairs(
        candidates("a.lisp", "(foo a) (foo b) (foo c)", 2),
        &SimilarityReportOptions {
            threshold: 0.0,
            overlap_policy: SimilarityOverlapPolicy::All,
            max_comparisons: Some(3),
            max_results: Some(1),
            ..SimilarityReportOptions::default()
        },
    );

    assert_eq!(report.summary.evaluated_pairs, 3);
    assert_eq!(report.summary.unprocessed_pairs, 0);
    assert!(!report.summary.comparison_limit_reached);
    assert_eq!(report.summary.matched_pairs, 3);
    assert_eq!(report.summary.reported_pairs, 1);
}

#[test]
fn max_comparisons_is_applied_after_size_pruning() {
    let values = candidates(
        "a.lisp",
        "(foo a) (foo a b c d) (bar x y z w) (baz p q r s)",
        2,
    );
    let report = super::reports::build_similarity_pairs(
        values,
        &SimilarityReportOptions {
            threshold: 0.6,
            overlap_policy: SimilarityOverlapPolicy::All,
            max_comparisons: Some(1),
            ..SimilarityReportOptions::default()
        },
    );

    assert_eq!(report.summary.possible_pairs, 6);
    assert_eq!(report.summary.evaluated_pairs, 1);
    assert_eq!(report.summary.pruned_by_size, 3);
    assert_eq!(report.summary.unprocessed_pairs, 2);
    assert!(report.summary.comparison_limit_reached);
}
