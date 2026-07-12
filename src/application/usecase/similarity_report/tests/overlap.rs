use super::*;

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
