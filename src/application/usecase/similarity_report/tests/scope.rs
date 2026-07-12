use super::*;

#[test]
fn form_scope_top_level_excludes_nested_forms() {
    let tree = SyntaxTree::parse("(outer (inner value))").unwrap();
    let mut values = Vec::new();
    collect_similarity_candidates(
        &tree,
        "(outer (inner value))",
        std::path::Path::new("a.lisp"),
        Dialect::CommonLisp,
        &SimilarityReportOptions {
            min_node_count: 2,
            form_scope: SimilarityFormScope::TopLevel,
            ..SimilarityReportOptions::default()
        },
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
        std::path::Path::new("a.lisp"),
        Dialect::CommonLisp,
        &SimilarityReportOptions {
            min_node_count: 2,
            min_line_span: 2,
            ..SimilarityReportOptions::default()
        },
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
        std::path::Path::new("a.lisp"),
        Dialect::CommonLisp,
        &SimilarityReportOptions {
            min_node_count: 3,
            min_line_span: 2,
            max_candidates: Some(1),
            ..SimilarityReportOptions::default()
        },
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
        std::path::Path::new("a.lisp"),
        Dialect::CommonLisp,
        &SimilarityReportOptions {
            min_node_count: 3,
            min_line_span: 2,
            max_candidates: Some(1),
            ..SimilarityReportOptions::default()
        },
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
            &SimilarityReportOptions {
                threshold: 0.0,
                comparison_scope,
                overlap_policy: SimilarityOverlapPolicy::All,
                ..SimilarityReportOptions::default()
            },
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
        crate::application::form_similarity::tree_similarity(&values[0].tree, &values[1].tree);
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
