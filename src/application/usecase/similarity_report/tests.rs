use std::path::Path;
use std::path::PathBuf;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::Path as ExpressionPath;
use crate::domain::sexpr::{ByteOffset, ByteSpan, SyntaxTree};

use super::*;

const DEFAULT_THRESHOLD: f64 = 0.87;
const DEFAULT_MIN_NODE_COUNT: usize = 4;
const DEFAULT_MIN_LINE_SPAN: usize = 1;
const DEFAULT_COMPARISON_SCOPE: SimilarityComparisonScope = SimilarityComparisonScope::All;
const DEFAULT_FORM_SCOPE: SimilarityFormScope = SimilarityFormScope::All;
const DEFAULT_OVERLAP_POLICY: SimilarityOverlapPolicy = SimilarityOverlapPolicy::Maximal;
const DEFAULT_MAX_CANDIDATES: Option<usize> = None;
const DEFAULT_MAX_COMPARISONS: Option<usize> = None;
const DEFAULT_MAX_RESULTS: Option<usize> = None;

fn options(
    threshold: f64,
    min_node_count: usize,
    min_line_span: usize,
    comparison_scope: SimilarityComparisonScope,
    form_scope: SimilarityFormScope,
    overlap_policy: SimilarityOverlapPolicy,
    max_candidates: Option<usize>,
    max_comparisons: Option<usize>,
    max_results: Option<usize>,
) -> SimilarityReportOptions {
    SimilarityReportOptions::new(
        threshold,
        min_node_count,
        min_line_span,
        comparison_scope,
        form_scope,
        overlap_policy,
        max_candidates,
        max_comparisons,
        max_results,
    )
    .unwrap()
}

fn candidates(file: &str, input: &str, min_node_count: usize) -> Vec<SimilarityCandidate> {
    let tree = SyntaxTree::parse(input).unwrap();
    let mut result = Vec::new();
    let report_options = options(
        DEFAULT_THRESHOLD,
        min_node_count,
        DEFAULT_MIN_LINE_SPAN,
        DEFAULT_COMPARISON_SCOPE,
        DEFAULT_FORM_SCOPE,
        DEFAULT_OVERLAP_POLICY,
        DEFAULT_MAX_CANDIDATES,
        DEFAULT_MAX_COMPARISONS,
        DEFAULT_MAX_RESULTS,
    );
    collect_similarity_candidates(
        &tree,
        input,
        Path::new(file),
        Dialect::CommonLisp,
        &report_options,
        &mut result,
    )
    .unwrap();
    result
}

fn build_similarity_pairs(
    candidates: Vec<SimilarityCandidate>,
    threshold: f64,
    overlap_policy: SimilarityOverlapPolicy,
    max_results: Option<usize>,
) -> SimilarityReport {
    super::reports::build_similarity_pairs(
        candidates,
        &options(
            threshold,
            DEFAULT_MIN_NODE_COUNT,
            DEFAULT_MIN_LINE_SPAN,
            DEFAULT_COMPARISON_SCOPE,
            DEFAULT_FORM_SCOPE,
            overlap_policy,
            DEFAULT_MAX_CANDIDATES,
            DEFAULT_MAX_COMPARISONS,
            max_results,
        ),
    )
}

fn report_form(path: &str, start: usize, end: usize) -> SimilarityFormReport {
    SimilarityFormReport {
        path: PathBuf::from(path),
        dialect: Dialect::CommonLisp,
        form_path: ExpressionPath::from_indexes(vec![start, end]),
        span: ByteSpan::new(ByteOffset::new(start), ByteOffset::new(end)),
        node_count: 1,
        head: None,
        text: String::new(),
    }
}

#[test]
fn form_scope_top_level_excludes_nested_forms() {
    let tree = SyntaxTree::parse("(outer (inner value))").unwrap();
    let mut values = Vec::new();
    collect_similarity_candidates(
        &tree,
        "(outer (inner value))",
        Path::new("a.lisp"),
        Dialect::CommonLisp,
        &options(
            DEFAULT_THRESHOLD,
            2,
            DEFAULT_MIN_LINE_SPAN,
            DEFAULT_COMPARISON_SCOPE,
            SimilarityFormScope::TopLevel,
            DEFAULT_OVERLAP_POLICY,
            DEFAULT_MAX_CANDIDATES,
            DEFAULT_MAX_COMPARISONS,
            DEFAULT_MAX_RESULTS,
        ),
        &mut values,
    )
    .unwrap();

    assert_eq!(values.len(), 1);
    assert_eq!(values[0].form.text, "(outer (inner value))");
}

#[test]
fn minimum_line_span_uses_inclusive_one_based_length() {
    let input = "(multi\n value)\n(single value)";
    let tree = SyntaxTree::parse(input).unwrap();
    let mut values = Vec::new();
    collect_similarity_candidates(
        &tree,
        input,
        Path::new("a.lisp"),
        Dialect::CommonLisp,
        &options(
            DEFAULT_THRESHOLD,
            2,
            2,
            DEFAULT_COMPARISON_SCOPE,
            DEFAULT_FORM_SCOPE,
            DEFAULT_OVERLAP_POLICY,
            DEFAULT_MAX_CANDIDATES,
            DEFAULT_MAX_COMPARISONS,
            DEFAULT_MAX_RESULTS,
        ),
        &mut values,
    )
    .unwrap();

    assert_eq!(values.len(), 1);
    assert_eq!(values[0].form.text, "(multi\n value)");
}

#[test]
fn candidate_limit_counts_only_eligible_omissions() {
    let input = "(keep value) (too-small) (omit\n value) (single-line value)";
    let tree = SyntaxTree::parse(input).unwrap();
    let mut values = Vec::new();
    let omitted = collect_similarity_candidates(
        &tree,
        input,
        Path::new("a.lisp"),
        Dialect::CommonLisp,
        &options(
            DEFAULT_THRESHOLD,
            3,
            2,
            DEFAULT_COMPARISON_SCOPE,
            DEFAULT_FORM_SCOPE,
            DEFAULT_OVERLAP_POLICY,
            Some(1),
            DEFAULT_MAX_COMPARISONS,
            DEFAULT_MAX_RESULTS,
        ),
        &mut values,
    )
    .unwrap();

    assert_eq!(values.len(), 1);
    assert_eq!(values[0].form.text, "(omit\n value)");
    assert_eq!(omitted, 0);

    let input = "(keep\n value) (too-small) (omit\n value) (single-line value)";
    let tree = SyntaxTree::parse(input).unwrap();
    values.clear();
    let omitted = collect_similarity_candidates(
        &tree,
        input,
        Path::new("a.lisp"),
        Dialect::CommonLisp,
        &options(
            DEFAULT_THRESHOLD,
            3,
            2,
            DEFAULT_COMPARISON_SCOPE,
            DEFAULT_FORM_SCOPE,
            DEFAULT_OVERLAP_POLICY,
            Some(1),
            DEFAULT_MAX_COMPARISONS,
            DEFAULT_MAX_RESULTS,
        ),
        &mut values,
    )
    .unwrap();

    assert_eq!(values.len(), 1);
    assert_eq!(values[0].form.text, "(keep\n value)");
    assert_eq!(omitted, 1);
}

#[test]
fn comparison_scope_filters_pair_population() {
    let mut values = candidates("a.lisp", "(foo a) (foo b)", 2);
    values.extend(candidates("b.lisp", "(foo c)", 2));
    let report = |comparison_scope| {
        super::reports::build_similarity_pairs(
            values.clone(),
            &options(
                0.0,
                DEFAULT_MIN_NODE_COUNT,
                DEFAULT_MIN_LINE_SPAN,
                comparison_scope,
                DEFAULT_FORM_SCOPE,
                SimilarityOverlapPolicy::All,
                DEFAULT_MAX_CANDIDATES,
                DEFAULT_MAX_COMPARISONS,
                DEFAULT_MAX_RESULTS,
            ),
        )
    };

    let all = report(SimilarityComparisonScope::All);
    let same_file = report(SimilarityComparisonScope::SameFile);
    let cross_file = report(SimilarityComparisonScope::CrossFile);
    assert_eq!(all.summary.possible_pairs, 3);
    assert_eq!(same_file.summary.possible_pairs, 1);
    assert_eq!(cross_file.summary.possible_pairs, 2);
    assert_eq!(same_file.summary.evaluated_pairs, 1);
    assert_eq!(cross_file.summary.evaluated_pairs, 2);
}

#[test]
fn threshold_is_inclusive() {
    let values = candidates("a.lisp", "(foo a b) (foo x y)", 2);
    let similarity =
        crate::domain::form_similarity::tree_similarity(&values[0].tree, &values[1].tree);
    let report = build_similarity_pairs(values, similarity, SimilarityOverlapPolicy::All, None);
    assert_eq!(report.pairs.len(), 1);
    assert_eq!(report.summary.evaluated_pairs, 1);
}

#[test]
fn empty_and_single_candidates_produce_no_pairs() {
    assert!(
        build_similarity_pairs(Vec::new(), 0.0, SimilarityOverlapPolicy::All, None)
            .pairs
            .is_empty()
    );
    assert!(build_similarity_pairs(
        candidates("a.lisp", "(foo a)", 2),
        0.0,
        SimilarityOverlapPolicy::All,
        None,
    )
    .pairs
    .is_empty());
}

#[test]
fn reversed_input_has_deterministic_output() {
    let mut forward = candidates("a.lisp", "(foo a b)", 2);
    forward.extend(candidates("b.lisp", "(foo x y)", 2));
    let mut reverse = forward.clone();
    reverse.reverse();
    assert_eq!(
        build_similarity_pairs(forward, 0.0, SimilarityOverlapPolicy::All, None),
        build_similarity_pairs(reverse, 0.0, SimilarityOverlapPolicy::All, None)
    );
}

#[test]
fn size_lower_bound_prunes_without_changing_inclusive_boundary() {
    let mut values = candidates("a.lisp", "(foo a)", 2);
    values.extend(candidates("b.lisp", "(foo a b c)", 2));

    let pruned = build_similarity_pairs(values.clone(), 0.75, SimilarityOverlapPolicy::All, None);
    assert_eq!(pruned.summary.possible_pairs, 1);
    assert_eq!(pruned.summary.pruned_by_size, 1);
    assert_eq!(pruned.summary.evaluated_pairs, 0);

    let boundary = build_similarity_pairs(values, 0.6, SimilarityOverlapPolicy::All, None);
    assert_eq!(boundary.summary.pruned_by_size, 0);
    assert_eq!(boundary.summary.evaluated_pairs, 1);

    let disabled = build_similarity_pairs(
        candidates("a.lisp", "(foo a) (foo a b c)", 2),
        0.0,
        SimilarityOverlapPolicy::All,
        None,
    );
    assert_eq!(disabled.summary.possible_pairs, 1);
    assert_eq!(disabled.summary.pruned_by_size, 0);
    assert_eq!(disabled.summary.evaluated_pairs, 1);
    assert_eq!(
        disabled.summary.evaluated_pairs + disabled.summary.pruned_by_size,
        disabled.summary.possible_pairs
    );
}

#[test]
fn sequential_size_pruning_keeps_later_valid_pairs() {
    let values = candidates("a.lisp", "(foo a b c d e) (foo a) (foo a b c d e)", 2);
    let report = super::reports::build_similarity_pairs(
        values,
        &options(
            0.75,
            DEFAULT_MIN_NODE_COUNT,
            DEFAULT_MIN_LINE_SPAN,
            DEFAULT_COMPARISON_SCOPE,
            DEFAULT_FORM_SCOPE,
            SimilarityOverlapPolicy::All,
            DEFAULT_MAX_CANDIDATES,
            Some(usize::MAX),
            DEFAULT_MAX_RESULTS,
        ),
    );

    assert_eq!(report.summary.possible_pairs, 3);
    assert_eq!(report.summary.pruned_by_size, 2);
    assert_eq!(report.summary.evaluated_pairs, 1);
    assert_eq!(report.pairs.len(), 1);
    assert_eq!(report.pairs[0].similarity, 1.0);
}

#[test]
fn maximal_overlap_suppresses_only_strictly_contained_pairs() {
    let mut values = candidates("a.lisp", "(outer (same x))", 2);
    values.extend(candidates("b.lisp", "(outer (same y))", 2));

    let all = build_similarity_pairs(values.clone(), 0.0, SimilarityOverlapPolicy::All, None);
    let maximal = build_similarity_pairs(values, 0.0, SimilarityOverlapPolicy::Maximal, None);
    assert_eq!(all.summary.suppressed_pairs, 0);
    assert!(maximal.summary.suppressed_pairs > 0);
    assert_eq!(maximal.summary.matched_pairs, all.summary.matched_pairs);

    let nested_pair = |report: &SimilarityReport| {
        report
            .pairs
            .iter()
            .any(|pair| pair.left.text == "(same x)" && pair.right.text == "(same y)")
    };
    assert!(nested_pair(&all));
    assert!(!nested_pair(&maximal));
}

#[test]
fn maximal_overlap_retains_sibling_pairs() {
    let mut pairs = vec![
        SimilarityPairReport {
            similarity: 1.0,
            score: 2.0,
            left: report_form("a.lisp", 0, 10),
            right: report_form("b.lisp", 0, 10),
        },
        SimilarityPairReport {
            similarity: 1.0,
            score: 1.0,
            left: report_form("a.lisp", 20, 30),
            right: report_form("b.lisp", 20, 30),
        },
    ];

    assert_eq!(super::reports::suppress_contained_pairs(&mut pairs), 0);
    assert_eq!(pairs.len(), 2);
}

#[test]
fn maximal_overlap_suppresses_when_only_one_side_is_strictly_contained() {
    let mut pairs = vec![
        SimilarityPairReport {
            similarity: 1.0,
            score: 2.0,
            left: report_form("a.lisp", 10, 20),
            right: report_form("b.lisp", 0, 30),
        },
        SimilarityPairReport {
            similarity: 1.0,
            score: 1.0,
            left: report_form("a.lisp", 0, 30),
            right: report_form("b.lisp", 0, 30),
        },
    ];

    assert_eq!(super::reports::suppress_contained_pairs(&mut pairs), 1);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].score, 1.0);
}

#[test]
fn maximal_overlap_suppression_is_independent_of_score_order() {
    let mut pairs = vec![
        SimilarityPairReport {
            similarity: 1.0,
            score: 10.0,
            left: report_form("a.lisp", 10, 20),
            right: report_form("b.lisp", 10, 20),
        },
        SimilarityPairReport {
            similarity: 1.0,
            score: 1.0,
            left: report_form("a.lisp", 0, 30),
            right: report_form("b.lisp", 0, 30),
        },
    ];

    assert_eq!(super::reports::suppress_contained_pairs(&mut pairs), 1);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].score, 1.0);
}

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
    assert_eq!(report.pairs[0].score, unlimited.pairs[0].score);
}

#[test]
fn max_comparisons_stops_ted_evaluation_and_tracks_unprocessed_pairs() {
    let report = super::reports::build_similarity_pairs(
        candidates("a.lisp", "(foo a) (foo b) (foo c)", 2),
        &options(
            0.0,
            DEFAULT_MIN_NODE_COUNT,
            DEFAULT_MIN_LINE_SPAN,
            DEFAULT_COMPARISON_SCOPE,
            DEFAULT_FORM_SCOPE,
            SimilarityOverlapPolicy::All,
            DEFAULT_MAX_CANDIDATES,
            Some(1),
            DEFAULT_MAX_RESULTS,
        ),
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
        &options(
            0.0,
            DEFAULT_MIN_NODE_COUNT,
            DEFAULT_MIN_LINE_SPAN,
            DEFAULT_COMPARISON_SCOPE,
            DEFAULT_FORM_SCOPE,
            SimilarityOverlapPolicy::All,
            DEFAULT_MAX_CANDIDATES,
            Some(3),
            Some(1),
        ),
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
        "(foo a) (foo a b c d) (foo x y z w) (foo p q r s)",
        2,
    );
    let report = super::reports::build_similarity_pairs(
        values,
        &options(
            0.6,
            DEFAULT_MIN_NODE_COUNT,
            DEFAULT_MIN_LINE_SPAN,
            DEFAULT_COMPARISON_SCOPE,
            DEFAULT_FORM_SCOPE,
            SimilarityOverlapPolicy::All,
            DEFAULT_MAX_CANDIDATES,
            Some(1),
            DEFAULT_MAX_RESULTS,
        ),
    );

    assert_eq!(report.summary.possible_pairs, 6);
    assert_eq!(report.summary.evaluated_pairs, 1);
    assert_eq!(report.summary.pruned_by_size, 3);
    assert_eq!(report.summary.unprocessed_pairs, 2);
    assert!(report.summary.comparison_limit_reached);
}

#[test]
fn different_heads_do_not_share_a_comparison_bucket() {
    let values = candidates("a.lisp", "(foo a b) (bar a b)", 2);
    let report = super::reports::build_similarity_pairs(
        values,
        &options(
            0.0,
            DEFAULT_MIN_NODE_COUNT,
            DEFAULT_MIN_LINE_SPAN,
            DEFAULT_COMPARISON_SCOPE,
            DEFAULT_FORM_SCOPE,
            SimilarityOverlapPolicy::All,
            DEFAULT_MAX_CANDIDATES,
            DEFAULT_MAX_COMPARISONS,
            DEFAULT_MAX_RESULTS,
        ),
    );

    assert_eq!(report.summary.possible_pairs, 0);
    assert_eq!(report.summary.evaluated_pairs, 0);
    assert!(report.pairs.is_empty());
}
