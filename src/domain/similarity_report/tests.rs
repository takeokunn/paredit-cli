use std::path::Path;
use std::path::PathBuf;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteOffset, ByteSpan, SyntaxTree};

use super::*;

fn candidates(file: &str, input: &str, min_node_count: usize) -> Vec<SimilarityCandidate> {
    let tree = SyntaxTree::parse(input).unwrap();
    let mut result = Vec::new();
    let options = report_options(
        0.87,
        min_node_count,
        1,
        SimilarityComparisonScope::All,
        SimilarityFormScope::All,
        SimilarityOverlapPolicy::Maximal,
        None,
        None,
        None,
    );
    collect_similarity_candidates(
        &tree,
        input,
        Path::new(file),
        Dialect::CommonLisp,
        &options,
        &mut result,
    )
    .unwrap();
    result
}

#[allow(clippy::too_many_arguments)]
fn report_options(
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

fn build_similarity_pairs(
    candidates: Vec<SimilarityCandidate>,
    threshold: f64,
    overlap_policy: SimilarityOverlapPolicy,
    max_results: Option<usize>,
) -> SimilarityReport {
    super::reports::build_similarity_pairs(
        candidates,
        &report_options(
            threshold,
            4,
            1,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            overlap_policy,
            None,
            None,
            max_results,
        ),
    )
    .unwrap()
}

fn report_form(path: &str, start: usize, end: usize) -> SimilarityFormReport {
    SimilarityFormReport {
        path: PathBuf::from(path),
        dialect: Dialect::CommonLisp,
        form_path: format!("{start}:{end}"),
        span: ByteSpan::new(ByteOffset::new(start), ByteOffset::new(end)),
        node_count: 1,
        head: None,
        text: String::new(),
    }
}

fn similarity_candidate(file: &str, node_count: usize) -> SimilarityCandidate {
    let input = "(foo a)";
    let tree = SyntaxTree::parse(input).unwrap();
    SimilarityCandidate {
        form: SimilarityFormReport {
            path: PathBuf::from(file),
            dialect: Dialect::CommonLisp,
            form_path: "0:7".to_string(),
            span: ByteSpan::new(ByteOffset::new(0), ByteOffset::new(input.len())),
            node_count,
            head: Some("foo".to_string()),
            text: input.to_string(),
        },
        tree: crate::domain::form_similarity::StructuralTree::from_view(
            &tree
                .select_path(&crate::domain::sexpr::Path::root_child(0))
                .unwrap()
                .view(),
        ),
        comparison_head: Some("foo".to_string()),
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
        &report_options(
            0.87,
            2,
            1,
            SimilarityComparisonScope::All,
            SimilarityFormScope::TopLevel,
            SimilarityOverlapPolicy::Maximal,
            None,
            None,
            None,
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
        &report_options(
            0.87,
            2,
            2,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            SimilarityOverlapPolicy::Maximal,
            None,
            None,
            None,
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
        &report_options(
            0.87,
            3,
            2,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            SimilarityOverlapPolicy::Maximal,
            Some(1),
            None,
            None,
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
        &report_options(
            0.87,
            3,
            2,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            SimilarityOverlapPolicy::Maximal,
            Some(1),
            None,
            None,
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
            &report_options(
                0.0,
                4,
                1,
                comparison_scope,
                SimilarityFormScope::All,
                SimilarityOverlapPolicy::All,
                None,
                None,
                None,
            ),
        )
        .unwrap()
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
    assert!(
        build_similarity_pairs(
            candidates("a.lisp", "(foo a)", 2),
            0.0,
            SimilarityOverlapPolicy::All,
            None,
        )
        .pairs
        .is_empty()
    );
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
        &report_options(
            0.75,
            4,
            1,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            SimilarityOverlapPolicy::All,
            None,
            Some(usize::MAX),
            None,
        ),
    )
    .unwrap();

    assert_eq!(report.summary.possible_pairs, 3);
    assert_eq!(report.summary.pruned_by_size, 2);
    assert_eq!(report.summary.evaluated_pairs, 1);
    assert_eq!(report.pairs.len(), 1);
    assert_eq!(report.pairs[0].similarity, 1.0);
}

#[test]
fn cross_file_size_pruning_keeps_later_valid_pairs() {
    let left_group = [similarity_candidate("a.lisp", 5)];
    let right_group = [
        similarity_candidate("b.lisp", 1),
        similarity_candidate("b.lisp", 5),
    ];
    let left_refs: Vec<_> = left_group.iter().collect();
    let right_refs: Vec<_> = right_group.iter().collect();
    let mut workspace = crate::domain::form_similarity::TreeSimilarityWorkspace::default();
    let mut evaluated_pairs = 0;

    let (output, limit_reached) = super::reports::compare_cross_file_group_pair(
        &left_refs,
        &right_refs,
        0.75,
        None,
        &mut evaluated_pairs,
        &mut workspace,
    );

    assert!(!limit_reached);
    assert_eq!(output.pruned_by_size, 1);
    assert_eq!(output.evaluated_pairs, 1);
    assert_eq!(output.pair_count(), 1);
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
        &report_options(
            0.0,
            4,
            1,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            SimilarityOverlapPolicy::All,
            None,
            Some(1),
            None,
        ),
    )
    .unwrap();

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
        &report_options(
            0.0,
            4,
            1,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            SimilarityOverlapPolicy::All,
            None,
            Some(3),
            Some(1),
        ),
    )
    .unwrap();

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
        &report_options(
            0.6,
            4,
            1,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            SimilarityOverlapPolicy::All,
            None,
            Some(1),
            None,
        ),
    )
    .unwrap();

    assert_eq!(report.summary.possible_pairs, 6);
    assert_eq!(report.summary.evaluated_pairs, 1);
    assert_eq!(report.summary.pruned_by_size, 3);
    assert_eq!(report.summary.unprocessed_pairs, 2);
    assert!(report.summary.comparison_limit_reached);
}

#[test]
fn candidate_limit_is_preserved_when_building_the_report() {
    let report = super::reports::build_similarity_pairs_with_omissions(
        candidates("a.lisp", "(foo a) (bar b) (baz c)", 2),
        1,
        &report_options(
            1.0,
            4,
            1,
            SimilarityComparisonScope::All,
            SimilarityFormScope::TopLevel,
            SimilarityOverlapPolicy::Maximal,
            None,
            None,
            None,
        ),
    )
    .unwrap();

    assert!(report.summary.candidate_limit_reached);
    assert_eq!(report.summary.omitted_candidates, 1);
}

#[test]
fn different_heads_do_not_share_a_comparison_bucket() {
    let values = candidates("a.lisp", "(foo a b) (bar a b)", 2);
    let report = super::reports::build_similarity_pairs(
        values,
        &report_options(
            0.0,
            4,
            1,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            SimilarityOverlapPolicy::All,
            None,
            None,
            None,
        ),
    )
    .unwrap();

    assert_eq!(report.summary.possible_pairs, 0);
    assert_eq!(report.summary.evaluated_pairs, 0);
    assert!(report.pairs.is_empty());
}
