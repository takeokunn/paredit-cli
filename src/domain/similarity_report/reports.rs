use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;
use std::path::Path;
use std::thread;

use crate::domain::form_similarity::{
    TreeSimilarityWorkspace, similarity_upper_bound, tree_similarity_with_workspace,
};
use anyhow::Result;

use super::types::{
    SimilarityCandidate, SimilarityComparisonScope, SimilarityFormReport, SimilarityOverlapPolicy,
    SimilarityPairReport, SimilarityReport, SimilarityReportOptions, SimilarityReportSummary,
    strictly_contains_pair_forms,
};

#[derive(Clone, Copy)]
struct SimilarityPairCandidate<'a> {
    similarity: f64,
    score: f64,
    left: &'a SimilarityCandidate,
    right: &'a SimilarityCandidate,
}

pub(super) trait PairLike {
    fn left_form(&self) -> &SimilarityFormReport;
    fn right_form(&self) -> &SimilarityFormReport;

    fn strictly_contains_pair(&self, other: &Self) -> bool {
        strictly_contains_pair_forms(
            self.left_form(),
            self.right_form(),
            other.left_form(),
            other.right_form(),
        )
    }
}

impl PairLike for SimilarityPairCandidate<'_> {
    fn left_form(&self) -> &SimilarityFormReport {
        &self.left.form
    }

    fn right_form(&self) -> &SimilarityFormReport {
        &self.right.form
    }
}

impl PairLike for SimilarityPairReport {
    fn left_form(&self) -> &SimilarityFormReport {
        &self.left
    }

    fn right_form(&self) -> &SimilarityFormReport {
        &self.right
    }
}

pub fn build_similarity_pairs(
    candidates: Vec<SimilarityCandidate>,
    options: &SimilarityReportOptions,
) -> Result<SimilarityReport> {
    build_similarity_pairs_with_omissions(candidates, 0, options)
}

/// Build the report while recording how many eligible candidates were dropped
/// upstream (e.g. by `--max-candidates`). Threading the count in here keeps the
/// summary correct by construction, instead of letting callers patch the
/// `candidate_limit_reached` / `omitted_candidates` fields after the fact.
pub fn build_similarity_pairs_with_omissions(
    mut candidates: Vec<SimilarityCandidate>,
    omitted_candidates: usize,
    options: &SimilarityReportOptions,
) -> Result<SimilarityReport> {
    options.validate()?;
    if options.max_comparisons().is_some() {
        return Ok(build_similarity_pairs_sequential(
            candidates,
            omitted_candidates,
            options,
        ));
    }
    // 低コストな候補から前に並べると、サイズ差だけで落ちる組を早く打ち切れる。
    candidates.sort_unstable_by(compare_candidates_for_scan);
    let possible_pairs = scoped_pair_count(&candidates, options.comparison_scope());
    let groups: Vec<&[SimilarityCandidate]> = candidates
        .chunk_by(SimilarityCandidate::same_comparison_bucket)
        .collect();
    let comparison_limit_reached = false;
    let mut evaluated_pairs = 0;
    let mut pruned_by_size = 0;
    let mut pairs: Vec<SimilarityPairCandidate<'_>> = Vec::new();
    if groups.is_empty() {
        return Ok(finalize_similarity_pairs(
            pairs,
            possible_pairs,
            evaluated_pairs,
            pruned_by_size,
            comparison_limit_reached,
            omitted_candidates,
            options,
        ));
    }

    let worker_count = thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1)
        .max(1);
    let work_items = build_work_items(&groups, options.comparison_scope(), worker_count);
    let worker_items = partition_items_for_workers(work_items, worker_count);

    thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in worker_items {
            handles.push(scope.spawn(move || {
                compare_work_items(&chunk, options.threshold(), options.comparison_scope())
            }));
        }

        for handle in handles {
            let output = handle
                .join()
                .expect("similarity comparison worker thread panicked");
            evaluated_pairs += output.evaluated_pairs;
            pruned_by_size += output.pruned_by_size;
            pairs.extend(output.pairs);
        }
    });

    Ok(finalize_similarity_pairs(
        pairs,
        possible_pairs,
        evaluated_pairs,
        pruned_by_size,
        comparison_limit_reached,
        omitted_candidates,
        options,
    ))
}

fn build_similarity_pairs_sequential(
    mut candidates: Vec<SimilarityCandidate>,
    omitted_candidates: usize,
    options: &SimilarityReportOptions,
) -> SimilarityReport {
    candidates.sort_unstable_by(compare_candidates_for_scan);
    let possible_pairs = scoped_pair_count(&candidates, options.comparison_scope());
    let mut comparison_limit_reached = false;
    let mut evaluated_pairs = 0;
    let mut pruned_by_size = 0;
    let mut pairs: Vec<SimilarityPairCandidate<'_>> = Vec::new();
    let mut workspace = TreeSimilarityWorkspace::default();
    'pairs: for group in candidates.chunk_by(SimilarityCandidate::same_comparison_bucket) {
        match options.comparison_scope() {
            SimilarityComparisonScope::All => {
                for left_index in 0..group.len() {
                    for right_index in left_index + 1..group.len() {
                        let left = &group[left_index];
                        let right = &group[right_index];
                        if size_bound_excludes(
                            left.form.node_count,
                            right.form.node_count,
                            options.threshold(),
                        ) {
                            pruned_by_size += 1;
                            if options.max_comparisons().is_none() {
                                break;
                            }
                            continue;
                        }
                        if options
                            .max_comparisons()
                            .is_some_and(|limit| evaluated_pairs == limit)
                        {
                            comparison_limit_reached = true;
                            break 'pairs;
                        }
                        evaluated_pairs += 1;
                        if similarity_upper_bound(&left.tree, &right.tree) < options.threshold() {
                            continue;
                        }
                        let similarity =
                            tree_similarity_with_workspace(&left.tree, &right.tree, &mut workspace);
                        if similarity >= options.threshold() {
                            let average_node_count =
                                (left.form.node_count + right.form.node_count) as f64 / 2.0;
                            pairs.push(SimilarityPairCandidate {
                                similarity,
                                score: similarity * average_node_count,
                                left,
                                right,
                            });
                        }
                    }
                }
            }
            SimilarityComparisonScope::SameFile => {
                let mut same_file_groups: HashMap<&Path, Vec<&SimilarityCandidate>> =
                    HashMap::with_capacity(group.len());
                for candidate in group {
                    same_file_groups
                        .entry(candidate.form.path.as_path())
                        .or_default()
                        .push(candidate);
                }

                for same_file_group in same_file_groups.values() {
                    for left_index in 0..same_file_group.len() {
                        for right_index in left_index + 1..same_file_group.len() {
                            let left = same_file_group[left_index];
                            let right = same_file_group[right_index];
                            if size_bound_excludes(
                                left.form.node_count,
                                right.form.node_count,
                                options.threshold(),
                            ) {
                                pruned_by_size += 1;
                                if options.max_comparisons().is_none() {
                                    break;
                                }
                                continue;
                            }
                            if options
                                .max_comparisons()
                                .is_some_and(|limit| evaluated_pairs == limit)
                            {
                                comparison_limit_reached = true;
                                break 'pairs;
                            }
                            evaluated_pairs += 1;
                            if similarity_upper_bound(&left.tree, &right.tree) < options.threshold()
                            {
                                continue;
                            }
                            let similarity = tree_similarity_with_workspace(
                                &left.tree,
                                &right.tree,
                                &mut workspace,
                            );
                            if similarity >= options.threshold() {
                                let average_node_count =
                                    (left.form.node_count + right.form.node_count) as f64 / 2.0;
                                pairs.push(SimilarityPairCandidate {
                                    similarity,
                                    score: similarity * average_node_count,
                                    left,
                                    right,
                                });
                            }
                        }
                    }
                }
            }
            SimilarityComparisonScope::CrossFile => {
                let mut cross_file_groups: HashMap<&Path, Vec<&SimilarityCandidate>> =
                    HashMap::with_capacity(group.len());
                for candidate in group {
                    cross_file_groups
                        .entry(candidate.form.path.as_path())
                        .or_default()
                        .push(candidate);
                }

                let cross_file_groups: Vec<_> = cross_file_groups.into_values().collect();
                'cross_file_groups: for left_group_index in 0..cross_file_groups.len() {
                    for right_group_index in left_group_index + 1..cross_file_groups.len() {
                        let (output, limit_reached) = compare_cross_file_group_pair(
                            cross_file_groups[left_group_index].as_slice(),
                            cross_file_groups[right_group_index].as_slice(),
                            options.threshold(),
                            options.max_comparisons(),
                            &mut evaluated_pairs,
                            &mut workspace,
                        );
                        pruned_by_size += output.pruned_by_size;
                        evaluated_pairs += output.evaluated_pairs;
                        pairs.extend(output.pairs);
                        if limit_reached {
                            comparison_limit_reached = true;
                            break 'cross_file_groups;
                        }
                    }
                }
            }
        }
    }

    finalize_similarity_pairs(
        pairs,
        possible_pairs,
        evaluated_pairs,
        pruned_by_size,
        comparison_limit_reached,
        omitted_candidates,
        options,
    )
}

fn finalize_similarity_pairs(
    mut pairs: Vec<SimilarityPairCandidate<'_>>,
    possible_pairs: usize,
    evaluated_pairs: usize,
    pruned_by_size: usize,
    comparison_limit_reached: bool,
    omitted_candidates: usize,
    options: &SimilarityReportOptions,
) -> SimilarityReport {
    let unprocessed_pairs = possible_pairs - evaluated_pairs - pruned_by_size;
    let matched_pairs = pairs.len();
    let suppressed_pairs = match options.overlap_policy() {
        SimilarityOverlapPolicy::All => 0,
        SimilarityOverlapPolicy::Maximal => suppress_contained_pairs(&mut pairs),
    };
    let truncated = options
        .max_results()
        .is_some_and(|limit| pairs.len() > limit);
    if let Some(limit) = options.max_results() {
        if limit == 0 {
            pairs.clear();
        } else if pairs.len() > limit {
            let nth = limit - 1;
            pairs.select_nth_unstable_by(nth, compare_pair_candidates);
            pairs[..limit].sort_unstable_by(compare_pair_candidates);
            pairs.truncate(limit);
        } else {
            pairs.sort_unstable_by(compare_pair_candidates);
        }
    } else {
        pairs.sort_unstable_by(compare_pair_candidates);
    }
    let reported_pairs = pairs.len();
    let pairs = pairs.into_iter().map(materialize_pair).collect();

    let summary = SimilarityReportSummary::new(
        omitted_candidates > 0,
        omitted_candidates,
        possible_pairs,
        evaluated_pairs,
        pruned_by_size,
        comparison_limit_reached,
        unprocessed_pairs,
        matched_pairs,
        suppressed_pairs,
        reported_pairs,
        truncated,
    );

    SimilarityReport::new(summary, pairs)
}

/// One schedulable unit of pair comparison. `Group` covers a whole
/// comparison bucket; `AllRange` covers only the left indexes in `left_range`
/// of one bucket under the `All` scope, so a single dominant bucket (for
/// example thousands of `defun` forms) can still spread across every worker
/// instead of pinning one thread while the rest idle.
enum WorkItem<'a> {
    Group(&'a [SimilarityCandidate]),
    AllRange {
        group: &'a [SimilarityCandidate],
        left_range: Range<usize>,
    },
}

/// Minimum estimated pair count before a bucket is worth splitting.
const SPLIT_MIN_PAIRS: usize = 2048;

fn build_work_items<'a>(
    groups: &[&'a [SimilarityCandidate]],
    scope: SimilarityComparisonScope,
    worker_count: usize,
) -> Vec<(WorkItem<'a>, usize)> {
    let mut items = Vec::new();
    for &group in groups {
        let cost = estimated_group_cost(group, scope);
        let splittable = matches!(scope, SimilarityComparisonScope::All)
            && worker_count > 1
            && cost > SPLIT_MIN_PAIRS;
        if !splittable {
            items.push((WorkItem::Group(group), cost));
            continue;
        }
        let chunk_count = (cost / SPLIT_MIN_PAIRS).clamp(1, worker_count * 4);
        for left_range in split_triangle_ranges(group.len(), chunk_count) {
            let chunk_cost = triangle_range_cost(group.len(), &left_range);
            items.push((WorkItem::AllRange { group, left_range }, chunk_cost));
        }
    }
    items
}

/// Splits the left indexes of a triangular all-pairs loop into `chunk_count`
/// ranges of roughly equal pair count. Index `i` contributes `len - 1 - i`
/// inner iterations, so equal-width ranges would front-load the work.
fn split_triangle_ranges(len: usize, chunk_count: usize) -> Vec<Range<usize>> {
    let total = pair_count(len);
    if chunk_count <= 1 || total == 0 {
        return std::iter::once(0..len).collect();
    }
    let target = total.div_ceil(chunk_count);
    let mut ranges = Vec::with_capacity(chunk_count);
    let mut start = 0;
    let mut accumulated = 0;
    for index in 0..len {
        accumulated += len - 1 - index;
        if accumulated >= target {
            ranges.push(start..index + 1);
            start = index + 1;
            accumulated = 0;
        }
    }
    if start < len {
        ranges.push(start..len);
    }
    ranges
}

fn triangle_range_cost(len: usize, left_range: &Range<usize>) -> usize {
    left_range.clone().map(|index| len - 1 - index).sum()
}

fn partition_items_for_workers(
    items: Vec<(WorkItem<'_>, usize)>,
    worker_count: usize,
) -> Vec<Vec<WorkItem<'_>>> {
    if worker_count <= 1 || items.len() <= 1 {
        return vec![items.into_iter().map(|(item, _)| item).collect()];
    }

    let mut weighted_items = items;
    weighted_items.sort_unstable_by_key(|item| std::cmp::Reverse(item.1));

    let mut assignments: Vec<(usize, Vec<WorkItem<'_>>)> =
        (0..worker_count).map(|_| (0, Vec::new())).collect();
    for (item, weight) in weighted_items {
        let target_index = assignments
            .iter()
            .enumerate()
            .min_by_key(|(_, (load, _))| *load)
            .map(|(index, _)| index)
            .unwrap();
        assignments[target_index].0 += weight;
        assignments[target_index].1.push(item);
    }

    assignments
        .into_iter()
        .filter_map(|(_, items)| (!items.is_empty()).then_some(items))
        .collect()
}

fn estimated_group_cost(group: &[SimilarityCandidate], scope: SimilarityComparisonScope) -> usize {
    match scope {
        SimilarityComparisonScope::All => pair_count(group.len()),
        SimilarityComparisonScope::SameFile => same_file_pair_count(group),
        SimilarityComparisonScope::CrossFile => {
            pair_count(group.len()) - same_file_pair_count(group)
        }
    }
}

fn scoped_pair_count(
    candidates: &[SimilarityCandidate],
    scope: SimilarityComparisonScope,
) -> usize {
    match scope {
        SimilarityComparisonScope::All => candidates
            .chunk_by(SimilarityCandidate::same_comparison_bucket)
            .map(|group| pair_count(group.len()))
            .sum(),
        SimilarityComparisonScope::SameFile => candidates
            .chunk_by(SimilarityCandidate::same_comparison_bucket)
            .map(same_file_pair_count)
            .sum(),
        SimilarityComparisonScope::CrossFile => candidates
            .chunk_by(SimilarityCandidate::same_comparison_bucket)
            .map(|group| pair_count(group.len()) - same_file_pair_count(group))
            .sum(),
    }
}

fn pair_count(count: usize) -> usize {
    count.saturating_sub(1) * count / 2
}

pub(crate) struct GroupComparisonOutput<'a> {
    pairs: Vec<SimilarityPairCandidate<'a>>,
    pub(crate) evaluated_pairs: usize,
    pub(crate) pruned_by_size: usize,
}

impl<'a> GroupComparisonOutput<'a> {
    #[cfg(test)]
    pub(crate) fn pair_count(&self) -> usize {
        self.pairs.len()
    }
}

fn compare_work_items<'a>(
    items: &[WorkItem<'a>],
    threshold: f64,
    scope: SimilarityComparisonScope,
) -> GroupComparisonOutput<'a> {
    let mut workspace = TreeSimilarityWorkspace::default();
    let mut output = GroupComparisonOutput {
        pairs: Vec::new(),
        evaluated_pairs: 0,
        pruned_by_size: 0,
    };
    for item in items {
        let item_output = match item {
            WorkItem::Group(group) => compare_group(group, threshold, scope, &mut workspace),
            WorkItem::AllRange { group, left_range } => {
                compare_group_all_range(group, left_range.clone(), threshold, &mut workspace)
            }
        };
        output.evaluated_pairs += item_output.evaluated_pairs;
        output.pruned_by_size += item_output.pruned_by_size;
        output.pairs.extend(item_output.pairs);
    }
    output
}

fn compare_group<'a>(
    group: &'a [SimilarityCandidate],
    threshold: f64,
    scope: SimilarityComparisonScope,
    workspace: &mut TreeSimilarityWorkspace,
) -> GroupComparisonOutput<'a> {
    match scope {
        SimilarityComparisonScope::All => {
            compare_group_all_range(group, 0..group.len(), threshold, workspace)
        }
        SimilarityComparisonScope::SameFile => compare_group_same_file(group, threshold, workspace),
        SimilarityComparisonScope::CrossFile => {
            compare_group_cross_file(group, threshold, workspace)
        }
    }
}

fn compare_group_all_range<'a>(
    group: &'a [SimilarityCandidate],
    left_range: Range<usize>,
    threshold: f64,
    workspace: &mut TreeSimilarityWorkspace,
) -> GroupComparisonOutput<'a> {
    let mut output = GroupComparisonOutput {
        pairs: Vec::new(),
        evaluated_pairs: 0,
        pruned_by_size: 0,
    };

    for left_index in left_range {
        for right_index in left_index + 1..group.len() {
            let left = &group[left_index];
            let right = &group[right_index];
            if size_bound_excludes(left.form.node_count, right.form.node_count, threshold) {
                output.pruned_by_size += 1;
                break;
            }

            output.evaluated_pairs += 1;
            if similarity_upper_bound(&left.tree, &right.tree) < threshold {
                continue;
            }
            let similarity = tree_similarity_with_workspace(&left.tree, &right.tree, workspace);
            if similarity >= threshold {
                let average_node_count =
                    (left.form.node_count + right.form.node_count) as f64 / 2.0;
                output.pairs.push(SimilarityPairCandidate {
                    similarity,
                    score: similarity * average_node_count,
                    left,
                    right,
                });
            }
        }
    }

    output
}

fn compare_group_same_file<'a>(
    group: &'a [SimilarityCandidate],
    threshold: f64,
    workspace: &mut TreeSimilarityWorkspace,
) -> GroupComparisonOutput<'a> {
    let mut output = GroupComparisonOutput {
        pairs: Vec::new(),
        evaluated_pairs: 0,
        pruned_by_size: 0,
    };
    let mut same_file_groups: HashMap<&Path, Vec<&SimilarityCandidate>> =
        HashMap::with_capacity(group.len());
    for candidate in group {
        same_file_groups
            .entry(candidate.form.path.as_path())
            .or_default()
            .push(candidate);
    }

    for same_file_group in same_file_groups.values() {
        for left_index in 0..same_file_group.len() {
            for right_index in left_index + 1..same_file_group.len() {
                let left = same_file_group[left_index];
                let right = same_file_group[right_index];
                if size_bound_excludes(left.form.node_count, right.form.node_count, threshold) {
                    output.pruned_by_size += 1;
                    break;
                }

                output.evaluated_pairs += 1;
                if similarity_upper_bound(&left.tree, &right.tree) < threshold {
                    continue;
                }
                let similarity = tree_similarity_with_workspace(&left.tree, &right.tree, workspace);
                if similarity >= threshold {
                    let average_node_count =
                        (left.form.node_count + right.form.node_count) as f64 / 2.0;
                    output.pairs.push(SimilarityPairCandidate {
                        similarity,
                        score: similarity * average_node_count,
                        left,
                        right,
                    });
                }
            }
        }
    }

    output
}

fn compare_group_cross_file<'a>(
    group: &'a [SimilarityCandidate],
    threshold: f64,
    workspace: &mut TreeSimilarityWorkspace,
) -> GroupComparisonOutput<'a> {
    let mut output = GroupComparisonOutput {
        pairs: Vec::new(),
        evaluated_pairs: 0,
        pruned_by_size: 0,
    };
    let mut cross_file_groups: HashMap<&Path, Vec<&SimilarityCandidate>> =
        HashMap::with_capacity(group.len());
    for candidate in group {
        cross_file_groups
            .entry(candidate.form.path.as_path())
            .or_default()
            .push(candidate);
    }

    let cross_file_groups: Vec<_> = cross_file_groups.into_values().collect();
    'cross_file_groups: for left_group_index in 0..cross_file_groups.len() {
        for right_group_index in left_group_index + 1..cross_file_groups.len() {
            let mut _pair_evaluated_pairs = 0;
            let (pair_output, limit_reached) = compare_cross_file_group_pair(
                cross_file_groups[left_group_index].as_slice(),
                cross_file_groups[right_group_index].as_slice(),
                threshold,
                None,
                &mut _pair_evaluated_pairs,
                workspace,
            );
            output.evaluated_pairs += pair_output.evaluated_pairs;
            output.pruned_by_size += pair_output.pruned_by_size;
            output.pairs.extend(pair_output.pairs);
            if limit_reached {
                break 'cross_file_groups;
            }
        }
    }

    output
}

pub(super) fn compare_cross_file_group_pair<'a>(
    left_group: &[&'a SimilarityCandidate],
    right_group: &[&'a SimilarityCandidate],
    threshold: f64,
    max_comparisons: Option<usize>,
    evaluated_pairs: &mut usize,
    workspace: &mut TreeSimilarityWorkspace,
) -> (GroupComparisonOutput<'a>, bool) {
    let mut output = GroupComparisonOutput {
        pairs: Vec::new(),
        evaluated_pairs: 0,
        pruned_by_size: 0,
    };

    for left in left_group {
        for right in right_group {
            if size_bound_excludes(left.form.node_count, right.form.node_count, threshold) {
                output.pruned_by_size += 1;
                continue;
            }

            if max_comparisons.is_some_and(|limit| *evaluated_pairs == limit) {
                return (output, true);
            }

            *evaluated_pairs += 1;
            output.evaluated_pairs += 1;
            if similarity_upper_bound(&left.tree, &right.tree) < threshold {
                continue;
            }
            let similarity = tree_similarity_with_workspace(&left.tree, &right.tree, workspace);
            if similarity >= threshold {
                let average_node_count =
                    (left.form.node_count + right.form.node_count) as f64 / 2.0;
                output.pairs.push(SimilarityPairCandidate {
                    similarity,
                    score: similarity * average_node_count,
                    left,
                    right,
                });
            }
        }
    }

    (output, false)
}

fn size_bound_excludes(left: usize, right: usize, threshold: f64) -> bool {
    let maximum = left.max(right) as f64;
    let difference = left.abs_diff(right) as f64;
    let allowed = (1.0 - threshold) * maximum;
    let tolerance = f64::EPSILON * maximum.max(1.0) * 4.0;
    difference > allowed + tolerance
}

fn same_file_pair_count(candidates: &[SimilarityCandidate]) -> usize {
    let mut counts: HashMap<&Path, usize> = HashMap::with_capacity(candidates.len());
    for candidate in candidates {
        *counts.entry(candidate.form.path.as_ref()).or_default() += 1;
    }
    counts.values().map(|&count| pair_count(count)).sum()
}

pub(super) fn suppress_contained_pairs<P: PairLike>(pairs: &mut Vec<P>) -> usize {
    let mut suppressed = vec![false; pairs.len()];
    {
        let mut groups: HashMap<(&Path, &Path), Vec<usize>> = HashMap::new();
        for (index, pair) in pairs.iter().enumerate() {
            groups
                .entry((
                    pair.left_form().path.as_ref(),
                    pair.right_form().path.as_ref(),
                ))
                .or_default()
                .push(index);
        }

        for indices in groups.values() {
            for (position, &index) in indices.iter().enumerate() {
                if suppressed[index] {
                    continue;
                }
                if indices[position + 1..]
                    .iter()
                    .any(|&other_index| pairs[other_index].strictly_contains_pair(&pairs[index]))
                    || indices[..position].iter().any(|&other_index| {
                        pairs[other_index].strictly_contains_pair(&pairs[index])
                    })
                {
                    suppressed[index] = true;
                }
            }
        }
    }
    let suppressed_count = suppressed.iter().filter(|&&value| value).count();
    let mut index = 0;
    pairs.retain(|_| {
        let retain = !suppressed[index];
        index += 1;
        retain
    });
    suppressed_count
}

fn compare_candidates_for_scan(
    left: &SimilarityCandidate,
    right: &SimilarityCandidate,
) -> Ordering {
    left.cmp_comparison_bucket(right)
        .then_with(|| left.form.node_count.cmp(&right.form.node_count))
        .then_with(|| left.form.path.as_os_str().cmp(right.form.path.as_os_str()))
        .then_with(|| left.form.form_path.cmp(&right.form.form_path))
}

fn compare_pair_candidates(
    left: &SimilarityPairCandidate<'_>,
    right: &SimilarityPairCandidate<'_>,
) -> Ordering {
    compare_pair_reports(left, right)
}

fn compare_pair_reports(left: &impl PairLikeScore, right: &impl PairLikeScore) -> Ordering {
    right
        .score()
        .total_cmp(&left.score())
        .then_with(|| right.similarity().total_cmp(&left.similarity()))
        .then_with(|| left.left_form().path.cmp(&right.left_form().path))
        .then_with(|| left.left_form().form_path.cmp(&right.left_form().form_path))
        .then_with(|| left.right_form().path.cmp(&right.right_form().path))
        .then_with(|| {
            left.right_form()
                .form_path
                .cmp(&right.right_form().form_path)
        })
}

trait PairLikeScore: PairLike {
    fn similarity(&self) -> f64;
    fn score(&self) -> f64;
}

impl PairLikeScore for SimilarityPairCandidate<'_> {
    fn similarity(&self) -> f64 {
        self.similarity
    }

    fn score(&self) -> f64 {
        self.score
    }
}

impl PairLikeScore for SimilarityPairReport {
    fn similarity(&self) -> f64 {
        self.similarity
    }

    fn score(&self) -> f64 {
        self.score
    }
}

fn materialize_pair(pair: SimilarityPairCandidate<'_>) -> SimilarityPairReport {
    SimilarityPairReport::new(
        pair.similarity,
        pair.score,
        pair.left.form.clone(),
        pair.right.form.clone(),
    )
}
