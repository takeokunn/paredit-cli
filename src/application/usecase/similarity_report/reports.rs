use std::cmp::Ordering;

use crate::application::form_similarity::tree_similarity;

use super::types::{
    SimilarityCandidate, SimilarityComparisonScope, SimilarityFormReport, SimilarityOverlapPolicy,
    SimilarityPairReport, SimilarityReport, SimilarityReportOptions, SimilarityReportSummary,
};

pub fn build_similarity_pairs(
    mut candidates: Vec<SimilarityCandidate>,
    options: &SimilarityReportOptions,
) -> SimilarityReport {
    candidates.sort_by(|left, right| form_key(left).cmp(&form_key(right)));
    let mut evaluated_pairs = 0;
    let mut pruned_by_size = 0;
    let mut pairs = Vec::new();
    for left_index in 0..candidates.len() {
        for right_index in left_index + 1..candidates.len() {
            let left = &candidates[left_index];
            let right = &candidates[right_index];
            if !comparison_is_in_scope(left, right, options.comparison_scope) {
                continue;
            }
            if size_bound_excludes(
                left.form.node_count,
                right.form.node_count,
                options.threshold,
            ) {
                pruned_by_size += 1;
                continue;
            }
            evaluated_pairs += 1;
            let similarity = tree_similarity(&left.tree, &right.tree);
            if similarity >= options.threshold {
                let average_node_count =
                    (left.form.node_count + right.form.node_count) as f64 / 2.0;
                pairs.push(SimilarityPairReport {
                    similarity,
                    score: similarity * average_node_count,
                    left: left.form.clone(),
                    right: right.form.clone(),
                });
            }
        }
    }
    pairs.sort_by(compare_pairs);
    let possible_pairs = evaluated_pairs + pruned_by_size;
    let matched_pairs = pairs.len();
    let suppressed_pairs = match options.overlap_policy {
        SimilarityOverlapPolicy::All => 0,
        SimilarityOverlapPolicy::Maximal => suppress_contained_pairs(&mut pairs),
    };
    let truncated = options.max_results.is_some_and(|limit| pairs.len() > limit);
    if let Some(limit) = options.max_results {
        pairs.truncate(limit);
    }
    let reported_pairs = pairs.len();

    SimilarityReport {
        summary: SimilarityReportSummary {
            possible_pairs,
            evaluated_pairs,
            pruned_by_size,
            matched_pairs,
            suppressed_pairs,
            reported_pairs,
            truncated,
        },
        pairs,
    }
}

fn comparison_is_in_scope(
    left: &SimilarityCandidate,
    right: &SimilarityCandidate,
    scope: SimilarityComparisonScope,
) -> bool {
    match scope {
        SimilarityComparisonScope::All => true,
        SimilarityComparisonScope::SameFile => left.form.path == right.form.path,
        SimilarityComparisonScope::CrossFile => left.form.path != right.form.path,
    }
}

fn size_bound_excludes(left: usize, right: usize, threshold: f64) -> bool {
    let maximum = left.max(right) as f64;
    let difference = left.abs_diff(right) as f64;
    let allowed = (1.0 - threshold) * maximum;
    let tolerance = f64::EPSILON * maximum.max(1.0) * 4.0;
    difference > allowed + tolerance
}

pub(super) fn suppress_contained_pairs(pairs: &mut Vec<SimilarityPairReport>) -> usize {
    let suppressed = (0..pairs.len())
        .map(|index| {
            (0..pairs.len()).any(|other_index| {
                index != other_index
                    && pair_is_strictly_contained(&pairs[index], &pairs[other_index])
            })
        })
        .collect::<Vec<_>>();
    let suppressed_count = suppressed.iter().filter(|&&value| value).count();
    let mut index = 0;
    pairs.retain(|_| {
        let retain = !suppressed[index];
        index += 1;
        retain
    });
    suppressed_count
}

fn pair_is_strictly_contained(lower: &SimilarityPairReport, higher: &SimilarityPairReport) -> bool {
    let left = form_is_contained(&lower.left, &higher.left);
    let right = form_is_contained(&lower.right, &higher.right);
    left.is_some_and(|strict| right.is_some_and(|other_strict| strict || other_strict))
}

fn form_is_contained(inner: &SimilarityFormReport, outer: &SimilarityFormReport) -> Option<bool> {
    if inner.path != outer.path
        || inner.span.start().get() < outer.span.start().get()
        || inner.span.end().get() > outer.span.end().get()
    {
        return None;
    }
    Some(inner.span != outer.span)
}

fn form_key(candidate: &SimilarityCandidate) -> (String, &str) {
    (
        candidate.form.path.display().to_string(),
        candidate.form.form_path.as_str(),
    )
}

fn compare_pairs(left: &SimilarityPairReport, right: &SimilarityPairReport) -> Ordering {
    right
        .score
        .total_cmp(&left.score)
        .then_with(|| right.similarity.total_cmp(&left.similarity))
        .then_with(|| left.left.path.cmp(&right.left.path))
        .then_with(|| left.left.form_path.cmp(&right.left.form_path))
        .then_with(|| left.right.path.cmp(&right.right.path))
        .then_with(|| left.right.form_path.cmp(&right.right.form_path))
}
