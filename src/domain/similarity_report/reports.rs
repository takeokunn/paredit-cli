use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering};
use std::thread;

use crate::domain::form_similarity::{
    MAX_REPORT_TREE_EDIT_OPERATIONS, MAX_TREE_SIMILARITY_WORKSPACES, TreeSimilarityError,
    TreeSimilarityOperationBudget, TreeSimilarityWorkspace, reserve_tree_similarity_workspaces,
    similarity_upper_bound, tree_similarity_with_workspace_and_budget,
};
use anyhow::{Result, anyhow};

use super::options::MAX_STORED_RESULTS;

use super::types::{
    SimilarityCandidate, SimilarityComparisonScope, SimilarityFormReport, SimilarityOverlapPolicy,
    SimilarityPairReport, SimilarityReport, SimilarityReportOptions, SimilarityReportSummary,
};

const MAX_SIMILARITY_WORKERS: usize = 8;
const DEFAULT_MAX_RESULTS: usize = 10_000;
const MAX_MATERIALIZED_RESULT_BYTES: usize = 64 * 1024 * 1024;

struct ResultBudget {
    retained: AtomicUsize,
    cancelled: AtomicBool,
    limit: usize,
    tree_edit_operations: TreeSimilarityOperationBudget,
}

impl ResultBudget {
    fn new() -> Self {
        Self {
            retained: AtomicUsize::new(0),
            cancelled: AtomicBool::new(false),
            limit: MAX_STORED_RESULTS,
            tree_edit_operations: TreeSimilarityOperationBudget::new(
                MAX_REPORT_TREE_EDIT_OPERATIONS,
            ),
        }
    }

    #[cfg(test)]
    fn with_limit(limit: usize) -> Self {
        Self {
            retained: AtomicUsize::new(0),
            cancelled: AtomicBool::new(false),
            limit,
            tree_edit_operations: TreeSimilarityOperationBudget::new(
                MAX_REPORT_TREE_EDIT_OPERATIONS,
            ),
        }
    }

    #[cfg(test)]
    fn with_limits(result_limit: usize, operation_limit: usize) -> Self {
        Self {
            retained: AtomicUsize::new(0),
            cancelled: AtomicBool::new(false),
            limit: result_limit,
            tree_edit_operations: TreeSimilarityOperationBudget::new(operation_limit),
        }
    }

    fn try_reserve(&self) -> bool {
        let reserved = self
            .retained
            .fetch_update(AtomicOrdering::Relaxed, AtomicOrdering::Relaxed, |count| {
                (count < self.limit).then_some(count + 1)
            })
            .is_ok();
        if !reserved {
            self.cancelled.store(true, AtomicOrdering::Release);
        }
        reserved
    }

    fn cancelled(&self) -> bool {
        self.cancelled.load(AtomicOrdering::Acquire)
    }

    fn comparison_cancelled(&self) -> bool {
        self.cancelled() || self.tree_edit_operations.exhausted()
    }

    fn tree_similarity(
        &self,
        left: &crate::domain::form_similarity::StructuralTree,
        right: &crate::domain::form_similarity::StructuralTree,
        workspace: &mut TreeSimilarityWorkspace,
    ) -> std::result::Result<f64, TreeSimilarityError> {
        tree_similarity_with_workspace_and_budget(
            left,
            right,
            workspace,
            Some(&self.tree_edit_operations),
        )
    }

    fn release(&self, count: usize) {
        if count == 0 {
            return;
        }
        let previous = self.retained.fetch_sub(count, AtomicOrdering::Relaxed);
        debug_assert!(previous >= count);
    }
}

#[derive(Clone, Copy)]
struct SimilarityPairCandidate<'a> {
    similarity: f64,
    score: f64,
    left: &'a SimilarityCandidate,
    right: &'a SimilarityCandidate,
}

#[derive(Clone, Copy)]
struct RankedPairCandidate<'a>(SimilarityPairCandidate<'a>);

impl PartialEq for RankedPairCandidate<'_> {
    fn eq(&self, other: &Self) -> bool {
        compare_pair_candidates(&self.0, &other.0).is_eq()
    }
}

impl Eq for RankedPairCandidate<'_> {}

impl PartialOrd for RankedPairCandidate<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RankedPairCandidate<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_pair_candidates(&self.0, &other.0)
    }
}

struct RetainedPairs<'a> {
    heap: BinaryHeap<RankedPairCandidate<'a>>,
}

impl<'a> RetainedPairs<'a> {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    fn len(&self) -> usize {
        self.heap.len()
    }

    fn push_reserved(&mut self, pair: SimilarityPairCandidate<'a>) {
        self.heap.push(RankedPairCandidate(pair));
    }

    fn replace_worst_if_better(&mut self, pair: SimilarityPairCandidate<'a>) {
        if self
            .heap
            .peek()
            .is_some_and(|worst| compare_pair_candidates(&pair, &worst.0).is_lt())
        {
            self.heap.pop();
            self.heap.push(RankedPairCandidate(pair));
        }
    }

    fn append(&mut self, other: &mut Self) {
        self.heap.append(&mut other.heap);
    }

    fn into_pairs(self) -> Vec<SimilarityPairCandidate<'a>> {
        self.heap.into_iter().map(|ranked| ranked.0).collect()
    }
}

struct ComparisonCounts {
    possible_pairs: usize,
    evaluated_pairs: usize,
    pruned_by_size: usize,
    resource_skipped_pairs: usize,
    matched_pairs: usize,
}

pub(super) trait PairLike {
    fn left_form(&self) -> &SimilarityFormReport;
    fn right_form(&self) -> &SimilarityFormReport;
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
    validate_candidate_budget(candidates.len())?;
    let result_budget = ResultBudget::new();
    if options.max_comparisons().is_some() {
        return build_similarity_pairs_sequential(
            candidates,
            omitted_candidates,
            options,
            &result_budget,
        );
    }
    // 低コストな候補から前に並べると、サイズ差だけで落ちる組を早く打ち切れる。
    candidates.sort_unstable_by(compare_candidates_for_scan);
    let possible_pairs = scoped_pair_count(&candidates, options.comparison_scope());
    validate_comparison_budget(possible_pairs, options.max_comparisons())?;
    let groups: Vec<&[SimilarityCandidate]> = candidates
        .chunk_by(SimilarityCandidate::same_comparison_bucket)
        .collect();
    let comparison_limit_reached = false;
    let mut evaluated_pairs: usize = 0;
    let mut pruned_by_size: usize = 0;
    let mut resource_skipped_pairs: usize = 0;
    let mut matched_pairs: usize = 0;
    let mut result_budget_exceeded = false;
    let mut pairs = RetainedPairs::new();
    if groups.is_empty() {
        return Ok(finalize_similarity_pairs(
            pairs.into_pairs(),
            ComparisonCounts {
                possible_pairs,
                evaluated_pairs,
                pruned_by_size,
                resource_skipped_pairs,
                matched_pairs,
            },
            comparison_limit_reached,
            omitted_candidates,
            options,
        ));
    }

    let available_workers = thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1)
        .max(1);
    let requested_workers = result_bounded_worker_count(
        effective_worker_count(available_workers, possible_pairs),
        collection_limit(options),
    );
    let workspace_reservation = reserve_tree_similarity_workspaces(requested_workers);
    let worker_count = workspace_reservation.count();
    let work_items = build_work_items(&groups, options.comparison_scope(), worker_count);
    let worker_items = partition_items_for_workers(work_items, worker_count)?;

    if !should_spawn_worker_threads(worker_items.len()) {
        let output = compare_work_items(
            &worker_items[0],
            options.threshold(),
            options.comparison_scope(),
            collection_limit(options),
            &result_budget,
        );
        evaluated_pairs = evaluated_pairs.saturating_add(output.evaluated_pairs);
        pruned_by_size = pruned_by_size.saturating_add(output.pruned_by_size);
        resource_skipped_pairs =
            resource_skipped_pairs.saturating_add(output.resource_skipped_pairs);
        merge_pairs(
            &mut pairs,
            &mut matched_pairs,
            &mut result_budget_exceeded,
            output,
            collection_limit(options),
            &result_budget,
        );
    } else {
        thread::scope(|scope| -> Result<()> {
            let mut handles = Vec::new();
            let result_budget = &result_budget;
            for chunk in worker_items {
                handles.push(scope.spawn(move || {
                    compare_work_items(
                        &chunk,
                        options.threshold(),
                        options.comparison_scope(),
                        collection_limit(options),
                        result_budget,
                    )
                }));
            }

            for handle in handles {
                let output = handle
                    .join()
                    .map_err(|_| anyhow!("similarity comparison worker thread panicked"))?;
                evaluated_pairs = evaluated_pairs.saturating_add(output.evaluated_pairs);
                pruned_by_size = pruned_by_size.saturating_add(output.pruned_by_size);
                resource_skipped_pairs =
                    resource_skipped_pairs.saturating_add(output.resource_skipped_pairs);
                merge_pairs(
                    &mut pairs,
                    &mut matched_pairs,
                    &mut result_budget_exceeded,
                    output,
                    collection_limit(options),
                    result_budget,
                );
            }
            Ok(())
        })?;
    }

    ensure_tree_edit_operation_budget(&result_budget)?;
    ensure_result_budget(result_budget_exceeded)?;

    Ok(finalize_similarity_pairs(
        pairs.into_pairs(),
        ComparisonCounts {
            possible_pairs,
            evaluated_pairs,
            pruned_by_size,
            resource_skipped_pairs,
            matched_pairs,
        },
        comparison_limit_reached,
        omitted_candidates,
        options,
    ))
}

fn build_similarity_pairs_sequential(
    mut candidates: Vec<SimilarityCandidate>,
    omitted_candidates: usize,
    options: &SimilarityReportOptions,
    result_budget: &ResultBudget,
) -> Result<SimilarityReport> {
    candidates.sort_unstable_by(compare_candidates_for_scan);
    let possible_pairs = scoped_pair_count(&candidates, options.comparison_scope());
    validate_comparison_budget(possible_pairs, options.max_comparisons())?;
    let mut comparison_limit_reached = false;
    let mut inspected_pairs: usize = 0;
    let mut evaluated_pairs: usize = 0;
    let mut pruned_by_size: usize = 0;
    let mut resource_skipped_pairs: usize = 0;
    let mut matched_pairs: usize = 0;
    let mut result_budget_exceeded = false;
    let mut pairs = RetainedPairs::new();
    let _workspace_reservation = reserve_tree_similarity_workspaces(1);
    let mut workspace = TreeSimilarityWorkspace::default();
    'pairs: for group in candidates.chunk_by(SimilarityCandidate::same_comparison_bucket) {
        match options.comparison_scope() {
            SimilarityComparisonScope::All => {
                for left_index in 0..group.len() {
                    for right_index in left_index + 1..group.len() {
                        if result_budget.comparison_cancelled() {
                            result_budget_exceeded |= result_budget.cancelled();
                            break 'pairs;
                        }
                        if options
                            .max_comparisons()
                            .is_some_and(|limit| inspected_pairs == limit)
                        {
                            comparison_limit_reached = true;
                            break 'pairs;
                        }
                        inspected_pairs = inspected_pairs.saturating_add(1);
                        let left = &group[left_index];
                        let right = &group[right_index];
                        if size_bound_excludes(
                            left.form.node_count,
                            right.form.node_count,
                            options.threshold(),
                        ) {
                            pruned_by_size = pruned_by_size.saturating_add(1);
                            continue;
                        }
                        evaluated_pairs = evaluated_pairs.saturating_add(1);
                        if similarity_upper_bound(&left.tree, &right.tree) < options.threshold() {
                            continue;
                        }
                        let Ok(similarity) =
                            result_budget.tree_similarity(&left.tree, &right.tree, &mut workspace)
                        else {
                            resource_skipped_pairs = resource_skipped_pairs.saturating_add(1);
                            continue;
                        };
                        if similarity >= options.threshold() {
                            let average_node_count =
                                average_node_count(left.form.node_count, right.form.node_count);
                            push_pair(
                                &mut pairs,
                                &mut matched_pairs,
                                &mut result_budget_exceeded,
                                SimilarityPairCandidate {
                                    similarity,
                                    score: similarity * average_node_count,
                                    left,
                                    right,
                                },
                                collection_limit(options),
                                result_budget,
                            );
                        }
                    }
                }
            }
            SimilarityComparisonScope::SameFile => {
                let mut same_file_groups: BTreeMap<&Path, Vec<&SimilarityCandidate>> =
                    BTreeMap::new();
                for candidate in group {
                    same_file_groups
                        .entry(candidate.form.path.as_path())
                        .or_default()
                        .push(candidate);
                }

                for same_file_group in same_file_groups.values() {
                    for left_index in 0..same_file_group.len() {
                        for right_index in left_index + 1..same_file_group.len() {
                            if result_budget.comparison_cancelled() {
                                result_budget_exceeded |= result_budget.cancelled();
                                break 'pairs;
                            }
                            if options
                                .max_comparisons()
                                .is_some_and(|limit| inspected_pairs == limit)
                            {
                                comparison_limit_reached = true;
                                break 'pairs;
                            }
                            inspected_pairs = inspected_pairs.saturating_add(1);
                            let left = same_file_group[left_index];
                            let right = same_file_group[right_index];
                            if size_bound_excludes(
                                left.form.node_count,
                                right.form.node_count,
                                options.threshold(),
                            ) {
                                pruned_by_size = pruned_by_size.saturating_add(1);
                                continue;
                            }
                            evaluated_pairs = evaluated_pairs.saturating_add(1);
                            if similarity_upper_bound(&left.tree, &right.tree) < options.threshold()
                            {
                                continue;
                            }
                            let Ok(similarity) = result_budget.tree_similarity(
                                &left.tree,
                                &right.tree,
                                &mut workspace,
                            ) else {
                                resource_skipped_pairs = resource_skipped_pairs.saturating_add(1);
                                continue;
                            };
                            if similarity >= options.threshold() {
                                let average_node_count =
                                    average_node_count(left.form.node_count, right.form.node_count);
                                push_pair(
                                    &mut pairs,
                                    &mut matched_pairs,
                                    &mut result_budget_exceeded,
                                    SimilarityPairCandidate {
                                        similarity,
                                        score: similarity * average_node_count,
                                        left,
                                        right,
                                    },
                                    collection_limit(options),
                                    result_budget,
                                );
                            }
                        }
                    }
                }
            }
            SimilarityComparisonScope::CrossFile => {
                let cross_file_groups = candidate_path_groups(group);
                let mut output = GroupComparisonOutput {
                    pairs: std::mem::replace(&mut pairs, RetainedPairs::new()),
                    matched_pairs,
                    result_budget_exceeded,
                    evaluated_pairs: 0,
                    pruned_by_size: 0,
                    resource_skipped_pairs: 0,
                };
                'cross_file_groups: for left_group_index in 0..cross_file_groups.len() {
                    for right_group_index in left_group_index + 1..cross_file_groups.len() {
                        let limit_reached = compare_cross_file_group_pair_into(
                            &mut output,
                            &cross_file_groups[left_group_index],
                            &cross_file_groups[right_group_index],
                            options.threshold(),
                            options.max_comparisons(),
                            collection_limit(options),
                            &mut inspected_pairs,
                            &mut workspace,
                            result_budget,
                        );
                        if result_budget.cancelled() {
                            output.result_budget_exceeded = true;
                            break 'cross_file_groups;
                        }
                        if result_budget.tree_edit_operations.exhausted() {
                            break 'cross_file_groups;
                        }
                        if limit_reached {
                            comparison_limit_reached = true;
                            break 'cross_file_groups;
                        }
                    }
                }
                pairs = output.pairs;
                matched_pairs = output.matched_pairs;
                result_budget_exceeded = output.result_budget_exceeded;
                evaluated_pairs = evaluated_pairs.saturating_add(output.evaluated_pairs);
                pruned_by_size = pruned_by_size.saturating_add(output.pruned_by_size);
                resource_skipped_pairs =
                    resource_skipped_pairs.saturating_add(output.resource_skipped_pairs);
                if result_budget.comparison_cancelled() {
                    break 'pairs;
                }
            }
        }
    }

    ensure_tree_edit_operation_budget(result_budget)?;
    ensure_result_budget(result_budget_exceeded)?;
    Ok(finalize_similarity_pairs(
        pairs.into_pairs(),
        ComparisonCounts {
            possible_pairs,
            evaluated_pairs,
            pruned_by_size,
            resource_skipped_pairs,
            matched_pairs,
        },
        comparison_limit_reached,
        omitted_candidates,
        options,
    ))
}

fn finalize_similarity_pairs(
    mut pairs: Vec<SimilarityPairCandidate<'_>>,
    counts: ComparisonCounts,
    comparison_limit_reached: bool,
    omitted_candidates: usize,
    options: &SimilarityReportOptions,
) -> SimilarityReport {
    let unprocessed_pairs = counts
        .possible_pairs
        .saturating_sub(counts.evaluated_pairs)
        .saturating_sub(counts.pruned_by_size);
    let suppressed_pairs = match options.overlap_policy() {
        SimilarityOverlapPolicy::All => 0,
        SimilarityOverlapPolicy::Maximal => suppress_contained_pairs(&mut pairs),
    };
    let pre_truncation_pairs = match options.overlap_policy() {
        SimilarityOverlapPolicy::All => counts.matched_pairs,
        SimilarityOverlapPolicy::Maximal => pairs.len(),
    };
    let limit = options.max_results().unwrap_or(DEFAULT_MAX_RESULTS);
    let mut truncated = pre_truncation_pairs > limit;
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
    let mut materialized_bytes = 0usize;
    let mut retained_by_bytes = 0usize;
    for pair in &pairs {
        let pair_bytes = pair
            .left
            .form
            .text
            .len()
            .saturating_add(pair.right.form.text.len());
        let Some(next) = materialized_bytes.checked_add(pair_bytes) else {
            break;
        };
        if next > MAX_MATERIALIZED_RESULT_BYTES {
            break;
        }
        materialized_bytes = next;
        retained_by_bytes += 1;
    }
    truncated |= retained_by_bytes < pairs.len();
    pairs.truncate(retained_by_bytes);
    let reported_pairs = pairs.len();
    let pairs = pairs.into_iter().map(materialize_pair).collect();

    let summary = SimilarityReportSummary::new(
        omitted_candidates > 0,
        omitted_candidates,
        counts.possible_pairs,
        counts.evaluated_pairs,
        counts.pruned_by_size,
        counts.resource_skipped_pairs,
        comparison_limit_reached,
        unprocessed_pairs,
        counts.matched_pairs,
        suppressed_pairs,
        reported_pairs,
        truncated,
    );

    SimilarityReport::new(summary, pairs)
}

/// One schedulable unit of pair comparison. Large buckets are divided along
/// the natural iteration boundary for each comparison scope.
enum WorkItem<'a> {
    Group(&'a [SimilarityCandidate]),
    AllRange {
        group: &'a [SimilarityCandidate],
        left_range: Range<usize>,
    },
    SameFileGroup(Arc<[&'a SimilarityCandidate]>),
    CrossFileRange {
        path_groups: Arc<[Arc<[&'a SimilarityCandidate]>]>,
        left_range: Range<usize>,
    },
}

/// Minimum estimated pair count before a bucket is worth splitting.
const SPLIT_MIN_PAIRS: usize = 2048;
const MAX_CANDIDATES: usize = 100_000;
const MAX_COMPARISONS: usize = 100_000_000;

fn validate_candidate_budget(candidate_count: usize) -> Result<()> {
    if candidate_count > MAX_CANDIDATES {
        return Err(anyhow!(
            "similarity candidate budget exceeded: {candidate_count} candidates, limit {MAX_CANDIDATES}"
        ));
    }
    Ok(())
}

fn validate_comparison_budget(possible_pairs: usize, max_comparisons: Option<usize>) -> Result<()> {
    let planned_comparisons = max_comparisons
        .map(|limit| limit.min(possible_pairs))
        .unwrap_or(possible_pairs);
    if planned_comparisons > MAX_COMPARISONS {
        return Err(anyhow!(
            "similarity comparison budget exceeded: {planned_comparisons} comparisons, limit {MAX_COMPARISONS}"
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn validate_resource_budgets_for_test(
    candidate_count: usize,
    possible_pairs: usize,
    max_comparisons: Option<usize>,
) -> Result<()> {
    validate_candidate_budget(candidate_count)?;
    validate_comparison_budget(possible_pairs, max_comparisons)
}

fn effective_worker_count(available_workers: usize, possible_pairs: usize) -> usize {
    if possible_pairs <= SPLIT_MIN_PAIRS {
        1
    } else {
        available_workers
            .clamp(1, MAX_SIMILARITY_WORKERS)
            .min(MAX_TREE_SIMILARITY_WORKSPACES)
    }
}

fn result_bounded_worker_count(requested_workers: usize, limit: Option<usize>) -> usize {
    let Some(limit) = limit else {
        return requested_workers;
    };
    if limit.saturating_mul(requested_workers) > MAX_STORED_RESULTS {
        1
    } else {
        requested_workers
    }
}

fn should_spawn_worker_threads(worker_item_count: usize) -> bool {
    worker_item_count > 1
}

#[cfg(test)]
pub(super) fn scheduling_policy_for_test(
    available_workers: usize,
    possible_pairs: usize,
    worker_item_count: usize,
) -> (usize, bool) {
    (
        effective_worker_count(available_workers, possible_pairs),
        should_spawn_worker_threads(worker_item_count),
    )
}

#[cfg(test)]
pub(super) fn result_bounded_worker_count_for_test(
    requested_workers: usize,
    limit: Option<usize>,
) -> usize {
    result_bounded_worker_count(requested_workers, limit)
}

fn build_work_items<'a>(
    groups: &[&'a [SimilarityCandidate]],
    scope: SimilarityComparisonScope,
    worker_count: usize,
) -> Vec<(WorkItem<'a>, usize)> {
    let mut items = Vec::new();
    for &group in groups {
        let cost = estimated_group_cost(group, scope);
        if worker_count <= 1 || cost <= SPLIT_MIN_PAIRS {
            items.push((WorkItem::Group(group), cost));
            continue;
        }

        match scope {
            SimilarityComparisonScope::All => {
                let chunk_count = (cost / SPLIT_MIN_PAIRS).clamp(1, worker_count.saturating_mul(4));
                for left_range in split_triangle_ranges(group.len(), chunk_count) {
                    let chunk_cost = triangle_range_cost(group.len(), &left_range);
                    items.push((WorkItem::AllRange { group, left_range }, chunk_cost));
                }
            }
            SimilarityComparisonScope::SameFile => {
                for path_group in candidate_path_groups(group) {
                    let path_cost = pair_count(path_group.len());
                    if path_cost > 0 {
                        items.push((WorkItem::SameFileGroup(path_group), path_cost));
                    }
                }
            }
            SimilarityComparisonScope::CrossFile => {
                let path_groups: Arc<[Arc<[&SimilarityCandidate]>]> =
                    Arc::from(candidate_path_groups(group));
                let chunk_count = worker_count.saturating_mul(4).min(path_groups.len()).max(1);
                for left_range in split_triangle_ranges(path_groups.len(), chunk_count) {
                    let chunk_cost = cross_file_range_cost(&path_groups, &left_range);
                    if chunk_cost == 0 {
                        continue;
                    }
                    items.push((
                        WorkItem::CrossFileRange {
                            path_groups: Arc::clone(&path_groups),
                            left_range,
                        },
                        chunk_cost,
                    ));
                }
            }
        }
    }
    items
}

#[cfg(test)]
pub(super) fn work_item_costs_for_test(
    groups: &[&[SimilarityCandidate]],
    scope: SimilarityComparisonScope,
    worker_count: usize,
) -> Vec<usize> {
    build_work_items(groups, scope, worker_count)
        .into_iter()
        .map(|(_, cost)| cost)
        .collect()
}

#[cfg(test)]
pub(super) fn tree_edit_operation_budget_execution_for_test(
    candidates: &[SimilarityCandidate],
    operation_limit: usize,
    worker_count: usize,
) -> (Option<String>, usize, bool, usize) {
    let groups = [candidates];
    let work_items = build_work_items(&groups, SimilarityComparisonScope::All, worker_count.max(1));
    let worker_items =
        partition_items_for_workers(work_items, worker_count.max(1)).expect("valid test workers");
    let spawned_worker_count = worker_items.len();
    let budget = ResultBudget::with_limits(MAX_STORED_RESULTS, operation_limit);

    if should_spawn_worker_threads(spawned_worker_count) {
        thread::scope(|scope| {
            let budget = &budget;
            worker_items
                .into_iter()
                .map(|items| {
                    scope.spawn(move || {
                        compare_work_items(
                            &items,
                            0.0,
                            SimilarityComparisonScope::All,
                            Some(0),
                            budget,
                        )
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|handle| {
                    handle.join().expect("test worker must not panic");
                });
        });
    } else {
        compare_work_items(
            &worker_items[0],
            0.0,
            SimilarityComparisonScope::All,
            Some(0),
            &budget,
        );
    }

    (
        ensure_tree_edit_operation_budget(&budget)
            .err()
            .map(|error| error.to_string()),
        budget.tree_edit_operations.operations(),
        budget.tree_edit_operations.exhausted(),
        spawned_worker_count,
    )
}

fn candidate_path_groups(group: &[SimilarityCandidate]) -> Vec<Arc<[&SimilarityCandidate]>> {
    let mut groups: BTreeMap<&Path, Vec<&SimilarityCandidate>> = BTreeMap::new();
    for candidate in group {
        groups
            .entry(candidate.form.path.as_path())
            .or_default()
            .push(candidate);
    }
    groups.into_values().map(Arc::from).collect()
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
    let mut accumulated = 0usize;
    for index in 0..len {
        accumulated = accumulated.saturating_add(len - 1 - index);
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
    left_range
        .clone()
        .fold(0, |cost, index| cost.saturating_add(len - 1 - index))
}

fn cross_file_range_cost(
    path_groups: &[Arc<[&SimilarityCandidate]>],
    left_range: &Range<usize>,
) -> usize {
    left_range.clone().fold(0, |cost, left_index| {
        (left_index + 1..path_groups.len()).fold(cost, |cost, right_index| {
            cost.saturating_add(
                path_groups[left_index]
                    .len()
                    .saturating_mul(path_groups[right_index].len()),
            )
        })
    })
}

fn partition_items_for_workers(
    items: Vec<(WorkItem<'_>, usize)>,
    worker_count: usize,
) -> Result<Vec<Vec<WorkItem<'_>>>> {
    if worker_count <= 1 || items.len() <= 1 {
        return Ok(vec![items.into_iter().map(|(item, _)| item).collect()]);
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
            .ok_or_else(|| anyhow!("similarity worker assignment has no available worker"))?;
        assignments[target_index].0 = assignments[target_index].0.saturating_add(weight);
        assignments[target_index].1.push(item);
    }

    Ok(assignments
        .into_iter()
        .filter_map(|(_, items)| (!items.is_empty()).then_some(items))
        .collect())
}

fn estimated_group_cost(group: &[SimilarityCandidate], scope: SimilarityComparisonScope) -> usize {
    match scope {
        SimilarityComparisonScope::All => pair_count(group.len()),
        SimilarityComparisonScope::SameFile => same_file_pair_count(group),
        SimilarityComparisonScope::CrossFile => {
            pair_count(group.len()).saturating_sub(same_file_pair_count(group))
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
            .fold(0, usize::saturating_add),
        SimilarityComparisonScope::SameFile => candidates
            .chunk_by(SimilarityCandidate::same_comparison_bucket)
            .map(same_file_pair_count)
            .fold(0, usize::saturating_add),
        SimilarityComparisonScope::CrossFile => candidates
            .chunk_by(SimilarityCandidate::same_comparison_bucket)
            .map(|group| pair_count(group.len()).saturating_sub(same_file_pair_count(group)))
            .fold(0, usize::saturating_add),
    }
}

fn pair_count(count: usize) -> usize {
    let previous = count.saturating_sub(1);
    if count % 2 == 0 {
        (count / 2).saturating_mul(previous)
    } else {
        count.saturating_mul(previous / 2)
    }
}

pub(crate) struct GroupComparisonOutput<'a> {
    pairs: RetainedPairs<'a>,
    matched_pairs: usize,
    result_budget_exceeded: bool,
    pub(crate) evaluated_pairs: usize,
    pub(crate) pruned_by_size: usize,
    pub(crate) resource_skipped_pairs: usize,
}

impl<'a> GroupComparisonOutput<'a> {
    fn new() -> Self {
        Self {
            pairs: RetainedPairs::new(),
            matched_pairs: 0,
            result_budget_exceeded: false,
            evaluated_pairs: 0,
            pruned_by_size: 0,
            resource_skipped_pairs: 0,
        }
    }

    fn push(
        &mut self,
        pair: SimilarityPairCandidate<'a>,
        limit: Option<usize>,
        budget: &ResultBudget,
    ) {
        push_pair(
            &mut self.pairs,
            &mut self.matched_pairs,
            &mut self.result_budget_exceeded,
            pair,
            limit,
            budget,
        );
    }
}

impl<'a> GroupComparisonOutput<'a> {
    #[cfg(test)]
    pub(crate) fn pair_count(&self) -> usize {
        self.pairs.len()
    }
}

fn collection_limit(options: &SimilarityReportOptions) -> Option<usize> {
    (options.overlap_policy() == SimilarityOverlapPolicy::All)
        .then(|| options.max_results().unwrap_or(DEFAULT_MAX_RESULTS))
}

fn push_pair<'a>(
    pairs: &mut RetainedPairs<'a>,
    matched_pairs: &mut usize,
    result_budget_exceeded: &mut bool,
    pair: SimilarityPairCandidate<'a>,
    limit: Option<usize>,
    budget: &ResultBudget,
) {
    *matched_pairs = matched_pairs.saturating_add(1);
    if limit == Some(0) {
        return;
    }
    if let Some(limit) = limit {
        if pairs.len() >= limit {
            pairs.replace_worst_if_better(pair);
            return;
        }
    }
    if !budget.try_reserve() {
        if limit.is_none() {
            *result_budget_exceeded = true;
        }
        return;
    }
    pairs.push_reserved(pair);
}

#[cfg(test)]
pub(super) fn bounded_result_scores_for_test(
    candidates: &[SimilarityCandidate],
    scores: &[f64],
    limit: usize,
) -> (usize, bool, bool, Vec<f64>) {
    assert!(candidates.len() >= 2);
    let budget = ResultBudget::with_limit(limit);
    let mut pairs = RetainedPairs::new();
    let mut matched_pairs = 0;
    let mut result_budget_exceeded = false;
    for &score in scores {
        push_pair(
            &mut pairs,
            &mut matched_pairs,
            &mut result_budget_exceeded,
            SimilarityPairCandidate {
                similarity: score,
                score,
                left: &candidates[0],
                right: &candidates[1],
            },
            Some(limit),
            &budget,
        );
    }
    let mut pairs = pairs.into_pairs();
    pairs.sort_unstable_by(compare_pair_candidates);
    (
        matched_pairs,
        result_budget_exceeded,
        budget.cancelled(),
        pairs.into_iter().map(|pair| pair.score).collect(),
    )
}

#[cfg(test)]
pub(super) fn bounded_result_retention_for_test(
    candidates: &[SimilarityCandidate],
    scores: &[f64],
    limit: usize,
) -> (usize, usize, usize, bool) {
    assert!(candidates.len() >= 2);
    let budget = ResultBudget::with_limit(limit);
    let mut pairs = RetainedPairs::new();
    let mut matched_pairs = 0;
    let mut result_budget_exceeded = false;
    let mut peak_retained = 0;
    for &score in scores {
        push_pair(
            &mut pairs,
            &mut matched_pairs,
            &mut result_budget_exceeded,
            SimilarityPairCandidate {
                similarity: score,
                score,
                left: &candidates[0],
                right: &candidates[1],
            },
            Some(limit),
            &budget,
        );
        peak_retained = peak_retained.max(pairs.len());
    }
    (
        matched_pairs,
        peak_retained,
        budget.retained.load(AtomicOrdering::Relaxed),
        budget.cancelled(),
    )
}

#[cfg(test)]
pub(super) fn bounded_result_paths_for_test(
    candidates: &[SimilarityCandidate],
    candidate_pairs: &[(usize, usize)],
    limit: usize,
) -> Vec<(String, String)> {
    let budget = ResultBudget::with_limit(limit);
    let mut pairs = RetainedPairs::new();
    let mut matched_pairs = 0;
    let mut result_budget_exceeded = false;
    for &(left, right) in candidate_pairs {
        push_pair(
            &mut pairs,
            &mut matched_pairs,
            &mut result_budget_exceeded,
            SimilarityPairCandidate {
                similarity: 1.0,
                score: 1.0,
                left: &candidates[left],
                right: &candidates[right],
            },
            Some(limit),
            &budget,
        );
    }
    let mut pairs = pairs.into_pairs();
    pairs.sort_unstable_by(compare_pair_candidates);
    pairs
        .into_iter()
        .map(|pair| {
            (
                pair.left.form.path.to_string_lossy().into_owned(),
                pair.right.form.path.to_string_lossy().into_owned(),
            )
        })
        .collect()
}

#[cfg(test)]
pub(super) fn bounded_parallel_result_paths_for_test(
    candidates: &[SimilarityCandidate],
    worker_pairs: &[&[(usize, usize)]],
    merge_order: &[usize],
    limit: usize,
) -> (bool, usize, usize, Vec<(String, String)>) {
    let budget = ResultBudget::with_limit(limit.saturating_mul(worker_pairs.len()));
    let mut outputs = thread::scope(|scope| {
        worker_pairs
            .iter()
            .map(|pairs| {
                scope.spawn(|| {
                    let mut output = GroupComparisonOutput::new();
                    for &(left, right) in *pairs {
                        output.push(
                            SimilarityPairCandidate {
                                similarity: 1.0,
                                score: 1.0,
                                left: &candidates[left],
                                right: &candidates[right],
                            },
                            Some(limit),
                            &budget,
                        );
                    }
                    output
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| Some(handle.join().expect("test worker must not panic")))
            .collect::<Vec<_>>()
    });
    let mut retained = RetainedPairs::new();
    let mut matched_pairs = 0;
    let mut result_budget_exceeded = false;
    for &worker_index in merge_order {
        merge_pairs(
            &mut retained,
            &mut matched_pairs,
            &mut result_budget_exceeded,
            outputs[worker_index].take().expect("worker merged once"),
            Some(limit),
            &budget,
        );
    }
    let retained_len = retained.len();
    let budget_retained = budget.retained.load(AtomicOrdering::Relaxed);
    let mut pairs = retained.into_pairs();
    pairs.sort_unstable_by(compare_pair_candidates);
    (
        budget.cancelled() || result_budget_exceeded,
        retained_len,
        budget_retained,
        pairs
            .into_iter()
            .map(|pair| {
                (
                    pair.left.form.path.to_string_lossy().into_owned(),
                    pair.right.form.path.to_string_lossy().into_owned(),
                )
            })
            .collect(),
    )
}

fn merge_pairs<'a>(
    pairs: &mut RetainedPairs<'a>,
    matched_pairs: &mut usize,
    result_budget_exceeded: &mut bool,
    mut other: GroupComparisonOutput<'a>,
    limit: Option<usize>,
    budget: &ResultBudget,
) {
    *matched_pairs = matched_pairs.saturating_add(other.matched_pairs);
    *result_budget_exceeded |= other.result_budget_exceeded;
    if let Some(limit) = limit {
        for ranked in other.pairs.heap.drain() {
            if pairs.len() < limit {
                pairs.heap.push(ranked);
            } else {
                pairs.replace_worst_if_better(ranked.0);
                budget.release(1);
            }
        }
    } else {
        pairs.append(&mut other.pairs);
    }
}

fn ensure_result_budget(exceeded: bool) -> Result<()> {
    if exceeded {
        return Err(anyhow!(
            "similarity result budget exceeded: more than {MAX_STORED_RESULTS} retained matches"
        ));
    }
    Ok(())
}

fn ensure_tree_edit_operation_budget(budget: &ResultBudget) -> Result<()> {
    if budget.tree_edit_operations.exhausted() {
        return Err(anyhow!(TreeSimilarityError::OperationBudgetExceeded {
            operations: budget.tree_edit_operations.operations().saturating_add(1),
            limit: budget.tree_edit_operations.limit(),
        }));
    }
    Ok(())
}

fn stop_for_budget(output: &mut GroupComparisonOutput<'_>, budget: &ResultBudget) -> bool {
    if !budget.comparison_cancelled() {
        return false;
    }
    output.result_budget_exceeded |= budget.cancelled();
    true
}

fn compare_work_items<'a>(
    items: &[WorkItem<'a>],
    threshold: f64,
    scope: SimilarityComparisonScope,
    result_limit: Option<usize>,
    budget: &ResultBudget,
) -> GroupComparisonOutput<'a> {
    let mut workspace = TreeSimilarityWorkspace::default();
    let mut output = GroupComparisonOutput::new();
    for item in items {
        if stop_for_budget(&mut output, budget) {
            break;
        }
        match item {
            WorkItem::Group(group) => compare_group_into(
                &mut output,
                group,
                threshold,
                scope,
                result_limit,
                &mut workspace,
                budget,
            ),
            WorkItem::AllRange { group, left_range } => compare_group_all_range_into(
                &mut output,
                group,
                left_range.clone(),
                threshold,
                result_limit,
                &mut workspace,
                budget,
            ),
            WorkItem::SameFileGroup(group) => compare_same_file_group_into(
                &mut output,
                group,
                threshold,
                result_limit,
                &mut workspace,
                budget,
            ),
            WorkItem::CrossFileRange {
                path_groups,
                left_range,
            } => compare_cross_file_range_into(
                &mut output,
                path_groups,
                left_range.clone(),
                threshold,
                result_limit,
                &mut workspace,
                budget,
            ),
        }
    }
    output
}

fn compare_cross_file_range_into<'a>(
    output: &mut GroupComparisonOutput<'a>,
    path_groups: &[Arc<[&'a SimilarityCandidate]>],
    left_range: Range<usize>,
    threshold: f64,
    result_limit: Option<usize>,
    workspace: &mut TreeSimilarityWorkspace,
    budget: &ResultBudget,
) {
    for left_index in left_range {
        if stop_for_budget(output, budget) {
            break;
        }
        for right_index in left_index + 1..path_groups.len() {
            if stop_for_budget(output, budget) {
                break;
            }
            let mut evaluated_pairs = 0;
            compare_cross_file_group_pair_into(
                output,
                &path_groups[left_index],
                &path_groups[right_index],
                threshold,
                None,
                result_limit,
                &mut evaluated_pairs,
                workspace,
                budget,
            );
        }
    }
}

fn compare_group_into<'a>(
    output: &mut GroupComparisonOutput<'a>,
    group: &'a [SimilarityCandidate],
    threshold: f64,
    scope: SimilarityComparisonScope,
    result_limit: Option<usize>,
    workspace: &mut TreeSimilarityWorkspace,
    budget: &ResultBudget,
) {
    match scope {
        SimilarityComparisonScope::All => compare_group_all_range_into(
            output,
            group,
            0..group.len(),
            threshold,
            result_limit,
            workspace,
            budget,
        ),
        SimilarityComparisonScope::SameFile => {
            compare_group_same_file_into(output, group, threshold, result_limit, workspace, budget)
        }
        SimilarityComparisonScope::CrossFile => {
            compare_group_cross_file_into(output, group, threshold, result_limit, workspace, budget)
        }
    }
}

fn compare_group_all_range_into<'a>(
    output: &mut GroupComparisonOutput<'a>,
    group: &'a [SimilarityCandidate],
    left_range: Range<usize>,
    threshold: f64,
    result_limit: Option<usize>,
    workspace: &mut TreeSimilarityWorkspace,
    budget: &ResultBudget,
) {
    for left_index in left_range {
        if stop_for_budget(output, budget) {
            break;
        }
        for right_index in left_index + 1..group.len() {
            if stop_for_budget(output, budget) {
                break;
            }
            let left = &group[left_index];
            let right = &group[right_index];
            if size_bound_excludes(left.form.node_count, right.form.node_count, threshold) {
                output.pruned_by_size = output.pruned_by_size.saturating_add(1);
                continue;
            }

            output.evaluated_pairs = output.evaluated_pairs.saturating_add(1);
            if similarity_upper_bound(&left.tree, &right.tree) < threshold {
                continue;
            }
            let Ok(similarity) = budget.tree_similarity(&left.tree, &right.tree, workspace) else {
                output.resource_skipped_pairs = output.resource_skipped_pairs.saturating_add(1);
                continue;
            };
            if similarity >= threshold {
                let average_node_count =
                    average_node_count(left.form.node_count, right.form.node_count);
                output.push(
                    SimilarityPairCandidate {
                        similarity,
                        score: similarity * average_node_count,
                        left,
                        right,
                    },
                    result_limit,
                    budget,
                );
            }
        }
    }
}

fn compare_group_same_file_into<'a>(
    output: &mut GroupComparisonOutput<'a>,
    group: &'a [SimilarityCandidate],
    threshold: f64,
    result_limit: Option<usize>,
    workspace: &mut TreeSimilarityWorkspace,
    budget: &ResultBudget,
) {
    let mut same_file_groups: BTreeMap<&Path, Vec<&SimilarityCandidate>> = BTreeMap::new();
    for candidate in group {
        same_file_groups
            .entry(candidate.form.path.as_path())
            .or_default()
            .push(candidate);
    }

    for same_file_group in same_file_groups.values() {
        compare_same_file_group_into(
            output,
            same_file_group,
            threshold,
            result_limit,
            workspace,
            budget,
        );
        if stop_for_budget(output, budget) {
            break;
        }
    }
}

fn compare_same_file_group_into<'a>(
    output: &mut GroupComparisonOutput<'a>,
    group: &[&'a SimilarityCandidate],
    threshold: f64,
    result_limit: Option<usize>,
    workspace: &mut TreeSimilarityWorkspace,
    budget: &ResultBudget,
) {
    for left_index in 0..group.len() {
        if stop_for_budget(output, budget) {
            break;
        }
        for right_index in left_index + 1..group.len() {
            if stop_for_budget(output, budget) {
                break;
            }
            let left = group[left_index];
            let right = group[right_index];
            if size_bound_excludes(left.form.node_count, right.form.node_count, threshold) {
                output.pruned_by_size = output.pruned_by_size.saturating_add(1);
                continue;
            }

            output.evaluated_pairs = output.evaluated_pairs.saturating_add(1);
            if similarity_upper_bound(&left.tree, &right.tree) < threshold {
                continue;
            }
            let Ok(similarity) = budget.tree_similarity(&left.tree, &right.tree, workspace) else {
                output.resource_skipped_pairs = output.resource_skipped_pairs.saturating_add(1);
                continue;
            };
            if similarity >= threshold {
                let average_node_count =
                    average_node_count(left.form.node_count, right.form.node_count);
                output.push(
                    SimilarityPairCandidate {
                        similarity,
                        score: similarity * average_node_count,
                        left,
                        right,
                    },
                    result_limit,
                    budget,
                );
            }
        }
    }
}

fn compare_group_cross_file_into<'a>(
    output: &mut GroupComparisonOutput<'a>,
    group: &'a [SimilarityCandidate],
    threshold: f64,
    result_limit: Option<usize>,
    workspace: &mut TreeSimilarityWorkspace,
    budget: &ResultBudget,
) {
    let mut cross_file_groups: BTreeMap<&Path, Vec<&SimilarityCandidate>> = BTreeMap::new();
    for candidate in group {
        cross_file_groups
            .entry(candidate.form.path.as_path())
            .or_default()
            .push(candidate);
    }

    let cross_file_groups: Vec<_> = cross_file_groups.into_values().collect();
    'cross_file_groups: for left_group_index in 0..cross_file_groups.len() {
        if stop_for_budget(output, budget) {
            break;
        }
        for right_group_index in left_group_index + 1..cross_file_groups.len() {
            let mut _pair_evaluated_pairs = 0;
            let limit_reached = compare_cross_file_group_pair_into(
                output,
                cross_file_groups[left_group_index].as_slice(),
                cross_file_groups[right_group_index].as_slice(),
                threshold,
                None,
                result_limit,
                &mut _pair_evaluated_pairs,
                workspace,
                budget,
            );
            if limit_reached || stop_for_budget(output, budget) {
                break 'cross_file_groups;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
fn compare_cross_file_group_pair_with_budget<'a>(
    left_group: &[&'a SimilarityCandidate],
    right_group: &[&'a SimilarityCandidate],
    threshold: f64,
    max_comparisons: Option<usize>,
    result_limit: Option<usize>,
    inspected_pairs: &mut usize,
    workspace: &mut TreeSimilarityWorkspace,
    budget: &ResultBudget,
) -> (GroupComparisonOutput<'a>, bool) {
    let mut output = GroupComparisonOutput::new();
    let limit_reached = compare_cross_file_group_pair_into(
        &mut output,
        left_group,
        right_group,
        threshold,
        max_comparisons,
        result_limit,
        inspected_pairs,
        workspace,
        budget,
    );
    (output, limit_reached)
}

#[allow(clippy::too_many_arguments)]
fn compare_cross_file_group_pair_into<'a>(
    output: &mut GroupComparisonOutput<'a>,
    left_group: &[&'a SimilarityCandidate],
    right_group: &[&'a SimilarityCandidate],
    threshold: f64,
    max_comparisons: Option<usize>,
    result_limit: Option<usize>,
    inspected_pairs: &mut usize,
    workspace: &mut TreeSimilarityWorkspace,
    budget: &ResultBudget,
) -> bool {
    for left in left_group {
        if stop_for_budget(output, budget) {
            return false;
        }
        for right in right_group {
            if stop_for_budget(output, budget) {
                return false;
            }
            if max_comparisons.is_some_and(|limit| *inspected_pairs == limit) {
                return true;
            }
            *inspected_pairs = inspected_pairs.saturating_add(1);

            if size_bound_excludes(left.form.node_count, right.form.node_count, threshold) {
                output.pruned_by_size = output.pruned_by_size.saturating_add(1);
                continue;
            }
            output.evaluated_pairs = output.evaluated_pairs.saturating_add(1);
            if similarity_upper_bound(&left.tree, &right.tree) < threshold {
                continue;
            }
            let Ok(similarity) = budget.tree_similarity(&left.tree, &right.tree, workspace) else {
                output.resource_skipped_pairs = output.resource_skipped_pairs.saturating_add(1);
                continue;
            };
            if similarity >= threshold {
                let average_node_count =
                    average_node_count(left.form.node_count, right.form.node_count);
                output.push(
                    SimilarityPairCandidate {
                        similarity,
                        score: similarity * average_node_count,
                        left,
                        right,
                    },
                    result_limit,
                    budget,
                );
            }
        }
    }

    false
}

#[cfg(test)]
pub(super) fn compare_cross_file_group_pair<'a>(
    left_group: &[&'a SimilarityCandidate],
    right_group: &[&'a SimilarityCandidate],
    threshold: f64,
    max_comparisons: Option<usize>,
    result_limit: Option<usize>,
    inspected_pairs: &mut usize,
    workspace: &mut TreeSimilarityWorkspace,
) -> (GroupComparisonOutput<'a>, bool) {
    compare_cross_file_group_pair_with_budget(
        left_group,
        right_group,
        threshold,
        max_comparisons,
        result_limit,
        inspected_pairs,
        workspace,
        &ResultBudget::new(),
    )
}

fn size_bound_excludes(left: usize, right: usize, threshold: f64) -> bool {
    let maximum = left.max(right) as f64;
    let difference = left.abs_diff(right) as f64;
    let allowed = (1.0 - threshold) * maximum;
    let tolerance = f64::EPSILON * maximum.max(1.0) * 4.0;
    difference > allowed + tolerance
}

fn average_node_count(left: usize, right: usize) -> f64 {
    left as f64 / 2.0 + right as f64 / 2.0
}

fn same_file_pair_count(candidates: &[SimilarityCandidate]) -> usize {
    let mut counts: HashMap<&Path, usize> = HashMap::with_capacity(candidates.len());
    for candidate in candidates {
        let count = counts.entry(candidate.form.path.as_ref()).or_default();
        *count = count.saturating_add(1);
    }
    counts
        .values()
        .map(|&count| pair_count(count))
        .fold(0, usize::saturating_add)
}

pub(super) fn suppress_contained_pairs<P: PairLike>(pairs: &mut Vec<P>) -> usize {
    let mut suppressed = vec![false; pairs.len()];
    {
        let mut groups: HashMap<(&Path, &Path), Vec<(usize, bool)>> = HashMap::new();
        for (index, pair) in pairs.iter().enumerate() {
            let left_form = pair.left_form();
            let right_form = pair.right_form();
            let left_path = left_form.path.as_ref();
            let right_path = right_form.path.as_ref();
            let swapped = left_path > right_path
                || (left_path == right_path
                    && compare_form_endpoints(left_form, right_form).is_gt());
            let paths = if swapped {
                (right_path, left_path)
            } else {
                (left_path, right_path)
            };
            groups.entry(paths).or_default().push((index, swapped));
        }

        for indices in groups.values() {
            suppress_group(pairs, indices, &mut suppressed);
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

type SpanKey = (usize, usize);

// Form spans from one syntax tree are nested or disjoint, so their containment
// relation can be represented as a forest.
struct SpanForest {
    nodes: Vec<SpanKey>,
    node_by_span: HashMap<SpanKey, usize>,
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
}

impl SpanForest {
    fn new(mut spans: Vec<SpanKey>) -> Self {
        spans.sort_unstable_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| right.1.cmp(&left.1))
        });
        spans.dedup();

        let mut nodes: Vec<SpanKey> = Vec::with_capacity(spans.len());
        let mut node_by_span = HashMap::with_capacity(spans.len());
        let mut children: Vec<Vec<usize>> = Vec::with_capacity(spans.len());
        let mut roots = Vec::new();
        let mut stack: Vec<usize> = Vec::new();

        for span in spans {
            while stack
                .last()
                .is_some_and(|&parent| !span_contains(nodes[parent], span))
            {
                stack.pop();
            }

            let node = nodes.len();
            nodes.push(span);
            node_by_span.insert(span, node);
            children.push(Vec::new());
            if let Some(&parent) = stack.last() {
                children[parent].push(node);
            } else {
                roots.push(node);
            }
            stack.push(node);
        }

        Self {
            nodes,
            node_by_span,
            children,
            roots,
        }
    }

    fn euler_ranges(&self) -> (Vec<usize>, Vec<usize>) {
        let mut starts = vec![0; self.nodes.len()];
        let mut ends = vec![0; self.nodes.len()];
        let mut clock = 0;
        let mut events = Vec::with_capacity(self.nodes.len().saturating_mul(2));
        for &root in self.roots.iter().rev() {
            events.push((root, false));
        }

        while let Some((node, exiting)) = events.pop() {
            if exiting {
                ends[node] = clock;
                continue;
            }
            starts[node] = clock;
            clock += 1;
            events.push((node, true));
            for &child in self.children[node].iter().rev() {
                events.push((child, false));
            }
        }
        (starts, ends)
    }
}

struct RangeCounter {
    tree: Vec<i64>,
}

impl RangeCounter {
    fn new(len: usize) -> Self {
        Self {
            tree: vec![0; len.saturating_add(1)],
        }
    }

    fn add(&mut self, index: usize, delta: i64) {
        let mut position = index + 1;
        while position < self.tree.len() {
            self.tree[position] += delta;
            position += position & position.wrapping_neg();
        }
    }

    fn add_range(&mut self, start: usize, end: usize, delta: i64) {
        self.add(start, delta);
        if end + 1 < self.tree.len() {
            self.add(end, -delta);
        }
    }

    fn at(&self, index: usize) -> i64 {
        let mut position = index + 1;
        let mut total = 0;
        while position > 0 {
            total += self.tree[position];
            position &= position - 1;
        }
        total
    }
}

fn suppress_group<P: PairLike>(pairs: &[P], indices: &[(usize, bool)], suppressed: &mut [bool]) {
    let left_forest = SpanForest::new(
        indices
            .iter()
            .map(|&(index, swapped)| form_span_key(oriented_forms(&pairs[index], swapped).0))
            .collect(),
    );
    let right_forest = SpanForest::new(
        indices
            .iter()
            .map(|&(index, swapped)| form_span_key(oriented_forms(&pairs[index], swapped).1))
            .collect(),
    );
    let (right_starts, right_ends) = right_forest.euler_ranges();
    let mut pairs_by_left = vec![Vec::new(); left_forest.nodes.len()];
    for &(index, swapped) in indices {
        let left_node =
            left_forest.node_by_span[&form_span_key(oriented_forms(&pairs[index], swapped).0)];
        pairs_by_left[left_node].push((index, swapped));
    }

    let mut active_right = RangeCounter::new(right_forest.nodes.len());
    let mut events = Vec::with_capacity(left_forest.nodes.len().saturating_mul(2));
    for &root in left_forest.roots.iter().rev() {
        events.push((root, false));
    }

    while let Some((left_node, exiting)) = events.pop() {
        let group = &pairs_by_left[left_node];
        if exiting {
            for &(index, swapped) in group {
                let right_node = right_forest.node_by_span
                    [&form_span_key(oriented_forms(&pairs[index], swapped).1)];
                active_right.add_range(right_starts[right_node], right_ends[right_node], -1);
            }
            continue;
        }

        let mut exact_counts = HashMap::with_capacity(group.len());
        for &(index, swapped) in group {
            let right_node =
                right_forest.node_by_span[&form_span_key(oriented_forms(&pairs[index], swapped).1)];
            *exact_counts.entry(right_node).or_insert(0_i64) += 1;
            active_right.add_range(right_starts[right_node], right_ends[right_node], 1);
        }
        for &(index, swapped) in group {
            let right_node =
                right_forest.node_by_span[&form_span_key(oriented_forms(&pairs[index], swapped).1)];
            suppressed[index] =
                active_right.at(right_starts[right_node]) > exact_counts[&right_node];
        }

        events.push((left_node, true));
        for &child in left_forest.children[left_node].iter().rev() {
            events.push((child, false));
        }
    }
}

fn oriented_forms<P: PairLike>(
    pair: &P,
    swapped: bool,
) -> (&SimilarityFormReport, &SimilarityFormReport) {
    if swapped {
        (pair.right_form(), pair.left_form())
    } else {
        (pair.left_form(), pair.right_form())
    }
}

fn compare_form_endpoints(left: &SimilarityFormReport, right: &SimilarityFormReport) -> Ordering {
    form_span_key(left)
        .cmp(&form_span_key(right))
        .then_with(|| left.form_path.cmp(&right.form_path))
}

fn form_span_key(form: &SimilarityFormReport) -> SpanKey {
    (form.span.start().get(), form.span.end().get())
}

fn span_contains(outer: SpanKey, inner: SpanKey) -> bool {
    outer.0 <= inner.0 && inner.1 <= outer.1
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
    SimilarityPairReport::from_shared(
        pair.similarity,
        pair.score,
        Arc::clone(&pair.left.form),
        Arc::clone(&pair.right.form),
    )
}

#[cfg(test)]
mod arithmetic_tests {
    use super::{ResultBudget, pair_count};

    #[test]
    fn pair_count_is_exact_for_representable_results() {
        assert_eq!(pair_count(0), 0);
        assert_eq!(pair_count(1), 0);
        assert_eq!(pair_count(2), 1);
        assert_eq!(pair_count(10), 45);
    }

    #[test]
    fn pair_count_saturates_instead_of_overflowing() {
        assert_eq!(pair_count(usize::MAX), usize::MAX);
        assert_eq!(pair_count(usize::MAX - 1), usize::MAX);
    }

    #[test]
    fn result_budget_reuses_released_capacity() {
        let budget = ResultBudget::with_limit(2);

        assert!(budget.try_reserve());
        assert!(budget.try_reserve());
        budget.release(1);
        assert!(budget.try_reserve());
        assert!(!budget.cancelled());
    }
}
