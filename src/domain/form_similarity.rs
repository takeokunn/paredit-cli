use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering};
use std::sync::{Condvar, Mutex};

use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, ReaderPrefix};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralTree {
    /// Order-sensitive digest of `labels`. Placed first so the derived
    /// `PartialEq` rejects unequal trees after one integer comparison instead
    /// of walking both label vectors.
    tree_hash: u64,
    labels: Vec<NodeLabel>,
    /// FNV-1a digest of each label, parallel to `labels`. Lets the edit
    /// distance inner loop and the multiset intersection compare labels
    /// without touching atom text.
    label_hashes: Vec<u64>,
    /// `label_hashes` sorted, for merge-based multiset intersection in
    /// `similarity_upper_bound`.
    sorted_label_hashes: Vec<u64>,
    leftmost: Vec<usize>,
    keyroots: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeLabel {
    Root(Vec<ReaderPrefix>),
    List(Option<Delimiter>, Vec<ReaderPrefix>),
    Atom(String, Vec<ReaderPrefix>),
}

const EDIT_COST_SCALE: usize = 10;
const ATOM_RENAME_COST: usize = 3;
const MAX_DISTANCE_MATRIX_CELLS: usize = 4 * 1024 * 1024;
const MAX_TREE_SIMILARITY_WORKSPACE_BYTES: usize = 64 * 1024 * 1024;
const MAX_TOTAL_TREE_SIMILARITY_WORKSPACE_BYTES: usize = 256 * 1024 * 1024;
pub(crate) const MAX_TREE_SIMILARITY_WORKSPACES: usize =
    MAX_TOTAL_TREE_SIMILARITY_WORKSPACE_BYTES / MAX_TREE_SIMILARITY_WORKSPACE_BYTES;
const MAX_TREE_EDIT_OPERATIONS: usize = 64 * 1024 * 1024;
pub(crate) const MAX_REPORT_TREE_EDIT_OPERATIONS: usize = MAX_TREE_EDIT_OPERATIONS;

struct TreeSimilarityWorkspaceLimiter {
    active: Mutex<usize>,
    available: Condvar,
    limit: usize,
}

impl TreeSimilarityWorkspaceLimiter {
    const fn new(limit: usize) -> Self {
        assert!(limit > 0);
        Self {
            active: Mutex::new(0),
            available: Condvar::new(),
            limit,
        }
    }

    fn acquire(&self, requested: usize) -> TreeSimilarityWorkspaceReservation<'_> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while *active >= self.limit {
            active = self
                .available
                .wait(active)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        let count = requested.max(1).min(self.limit.saturating_sub(*active));
        *active = active.saturating_add(count);
        TreeSimilarityWorkspaceReservation {
            limiter: self,
            count,
        }
    }
}

pub(crate) struct TreeSimilarityWorkspaceReservation<'a> {
    limiter: &'a TreeSimilarityWorkspaceLimiter,
    count: usize,
}

impl TreeSimilarityWorkspaceReservation<'_> {
    pub(crate) const fn count(&self) -> usize {
        self.count
    }
}

impl Drop for TreeSimilarityWorkspaceReservation<'_> {
    fn drop(&mut self) {
        let mut active = self
            .limiter
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *active = active.saturating_sub(self.count);
        self.limiter.available.notify_all();
    }
}

static TREE_SIMILARITY_WORKSPACE_LIMITER: TreeSimilarityWorkspaceLimiter =
    TreeSimilarityWorkspaceLimiter::new(MAX_TREE_SIMILARITY_WORKSPACES);

pub(crate) fn reserve_tree_similarity_workspaces(
    requested: usize,
) -> TreeSimilarityWorkspaceReservation<'static> {
    TREE_SIMILARITY_WORKSPACE_LIMITER.acquire(requested)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSimilarityError {
    MatrixTooLarge { cells: usize, bytes: usize },
    AllocationFailed { cells: usize, bytes: usize },
    OperationBudgetExceeded { operations: usize, limit: usize },
}

impl fmt::Display for TreeSimilarityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MatrixTooLarge { cells, bytes } => write!(
                formatter,
                "tree similarity matrix exceeds resource budget ({cells} cells, {bytes} bytes)"
            ),
            Self::AllocationFailed { cells, bytes } => write!(
                formatter,
                "tree similarity matrix allocation failed ({cells} cells, {bytes} bytes)"
            ),
            Self::OperationBudgetExceeded { operations, limit } => write!(
                formatter,
                "tree similarity operation budget exceeded ({operations} operations, limit {limit})"
            ),
        }
    }
}

impl Error for TreeSimilarityError {}

#[derive(Debug, Default)]
pub(crate) struct TreeSimilarityWorkspace {
    tree_distances: Vec<usize>,
    forest_distances: Vec<usize>,
}

#[derive(Debug)]
pub(crate) struct TreeSimilarityOperationBudget {
    operations: AtomicUsize,
    exhausted: AtomicBool,
    limit: usize,
}

impl TreeSimilarityOperationBudget {
    pub(crate) const fn new(limit: usize) -> Self {
        Self {
            operations: AtomicUsize::new(0),
            exhausted: AtomicBool::new(false),
            limit,
        }
    }

    #[inline]
    fn consume(&self) -> Result<(), TreeSimilarityError> {
        match self.operations.fetch_update(
            AtomicOrdering::AcqRel,
            AtomicOrdering::Acquire,
            |operations| operations.checked_add(1).filter(|&next| next <= self.limit),
        ) {
            Ok(_) => Ok(()),
            Err(operations) => {
                self.exhausted.store(true, AtomicOrdering::Release);
                Err(TreeSimilarityError::OperationBudgetExceeded {
                    operations: operations.saturating_add(1),
                    limit: self.limit,
                })
            }
        }
    }

    pub(crate) fn exhausted(&self) -> bool {
        self.exhausted.load(AtomicOrdering::Acquire)
    }

    pub(crate) fn operations(&self) -> usize {
        self.operations.load(AtomicOrdering::Acquire)
    }

    pub(crate) const fn limit(&self) -> usize {
        self.limit
    }
}

impl TreeSimilarityWorkspace {
    fn try_reset(&mut self, len: usize, bytes: usize) -> Result<(), TreeSimilarityError> {
        if self.tree_distances.capacity() >= len && self.forest_distances.capacity() >= len {
            self.tree_distances.resize(len, 0);
            self.forest_distances.resize(len, 0);
            self.tree_distances.fill(0);
            self.forest_distances.fill(0);
            return Ok(());
        }

        // Allocate transactionally so a failure for the second matrix does not
        // leave the reusable workspace retaining the first large allocation.
        let mut tree_distances = Vec::new();
        tree_distances
            .try_reserve_exact(len)
            .map_err(|_| TreeSimilarityError::AllocationFailed { cells: len, bytes })?;
        let mut forest_distances = Vec::new();
        forest_distances
            .try_reserve_exact(len)
            .map_err(|_| TreeSimilarityError::AllocationFailed { cells: len, bytes })?;
        tree_distances.resize(len, 0);
        forest_distances.resize(len, 0);
        self.tree_distances = tree_distances;
        self.forest_distances = forest_distances;
        Ok(())
    }
}

impl StructuralTree {
    pub fn from_view(view: &ExpressionView) -> Self {
        Self::from_view_with_count(view).0
    }

    pub(crate) fn from_view_with_count(view: &ExpressionView) -> (Self, usize) {
        fn label(view: &ExpressionView) -> NodeLabel {
            match view.kind {
                ExpressionKind::Root => NodeLabel::Root(view.reader_prefixes.clone()),
                ExpressionKind::List => {
                    NodeLabel::List(view.delimiter, view.reader_prefixes.clone())
                }
                ExpressionKind::Atom => NodeLabel::Atom(
                    view.text.clone().unwrap_or_default(),
                    view.reader_prefixes.clone(),
                ),
            }
        }

        enum Visit<'a> {
            Enter(&'a ExpressionView),
            Exit {
                view: &'a ExpressionView,
                descendant_start: usize,
            },
        }

        let mut labels = Vec::new();
        let mut leftmost = Vec::new();
        let mut pending = vec![Visit::Enter(view)];
        while let Some(frame) = pending.pop() {
            match frame {
                Visit::Enter(view) => {
                    pending.push(Visit::Exit {
                        view,
                        descendant_start: labels.len(),
                    });
                    pending.extend(view.children.iter().rev().map(Visit::Enter));
                }
                Visit::Exit {
                    view,
                    descendant_start,
                } => {
                    let index = labels.len() + 1;
                    let leaf = leftmost.get(descendant_start).copied().unwrap_or(index);
                    labels.push(label(view));
                    leftmost.push(leaf);
                }
            }
        }
        let node_count = labels.len();
        let mut keyroots = vec![0; node_count + 1];
        for (offset, leaf) in leftmost.iter().copied().enumerate() {
            keyroots[leaf] = offset + 1;
        }
        let mut keyroots = keyroots
            .into_iter()
            .skip(1)
            .filter(|&index| index != 0)
            .collect::<Vec<_>>();
        keyroots.sort_unstable();

        let label_hashes: Vec<u64> = labels.iter().map(hash_label).collect();
        let mut sorted_label_hashes = label_hashes.clone();
        sorted_label_hashes.sort_unstable();
        let tree_hash = label_hashes
            .iter()
            .fold(FNV_OFFSET, |hash, &label| fnv_u64(hash, label));

        (
            Self {
                tree_hash,
                labels,
                label_hashes,
                sorted_label_hashes,
                leftmost,
                keyroots,
            },
            node_count,
        )
    }

    pub fn node_count(&self) -> usize {
        self.labels.len()
    }
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[inline]
fn fnv_byte(hash: u64, byte: u8) -> u64 {
    (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
}

#[inline]
fn fnv_u64(hash: u64, value: u64) -> u64 {
    value
        .to_le_bytes()
        .iter()
        .fold(hash, |hash, &byte| fnv_byte(hash, byte))
}

fn hash_label(label: &NodeLabel) -> u64 {
    fn hash_prefixes(mut hash: u64, prefixes: &[ReaderPrefix]) -> u64 {
        for prefix in prefixes {
            hash = fnv_byte(hash, *prefix as u8 + 1);
        }
        hash
    }

    match label {
        NodeLabel::Root(prefixes) => hash_prefixes(fnv_byte(FNV_OFFSET, 0), prefixes),
        NodeLabel::List(delimiter, prefixes) => {
            let delimiter_byte = match delimiter {
                None => 0,
                Some(Delimiter::Paren) => 1,
                Some(Delimiter::Bracket) => 2,
                Some(Delimiter::Brace) => 3,
            };
            hash_prefixes(fnv_byte(fnv_byte(FNV_OFFSET, 1), delimiter_byte), prefixes)
        }
        NodeLabel::Atom(text, prefixes) => {
            let mut hash = fnv_byte(FNV_OFFSET, 2);
            for byte in text.bytes() {
                hash = fnv_byte(hash, byte);
            }
            // Terminator keeps `("ab", [Quote])` and `("ab\x01", [])`-style
            // field boundaries from colliding.
            hash_prefixes(fnv_byte(hash, 0xff), prefixes)
        }
    }
}

/// Counts the multiset intersection of two sorted hash sequences by merging.
fn sorted_intersection_count(left: &[u64], right: &[u64]) -> usize {
    let mut shared = 0;
    let mut left_index = 0;
    let mut right_index = 0;
    while left_index < left.len() && right_index < right.len() {
        match left[left_index].cmp(&right[right_index]) {
            Ordering::Less => left_index += 1,
            Ordering::Greater => right_index += 1,
            Ordering::Equal => {
                shared += 1;
                left_index += 1;
                right_index += 1;
            }
        }
    }
    shared
}

/// Cheap upper bound on `tree_similarity` derived from label multisets.
///
/// Any edit mapping matches `k <= min(n, m)` node pairs, deletes the other
/// `n - k` left nodes, and inserts the other `m - k` right nodes. Matched
/// pairs with identical labels cost 0, and there can be at most
/// `|multiset intersection|` of those; every other matched pair costs at
/// least `ATOM_RENAME_COST`. The bound below minimizes that total over `k`,
/// so the true edit distance can never be smaller and the true similarity
/// can never be larger. Distinct labels that collide in the hash only make
/// the intersection look bigger, which loosens the bound but keeps it sound.
pub(crate) fn similarity_upper_bound(left: &StructuralTree, right: &StructuralTree) -> f64 {
    let left_count = left.labels.len();
    let right_count = right.labels.len();
    let matched = left_count.min(right_count);
    let shared = sorted_intersection_count(&left.sorted_label_hashes, &right.sorted_label_hashes);
    let lower_bound_scaled = EDIT_COST_SCALE as f64 * left_count.abs_diff(right_count) as f64
        + ATOM_RENAME_COST as f64 * matched.saturating_sub(shared) as f64;
    1.0 - lower_bound_scaled / (EDIT_COST_SCALE as f64 * left_count.max(right_count) as f64)
}

pub fn tree_similarity(
    left: &StructuralTree,
    right: &StructuralTree,
) -> Result<f64, TreeSimilarityError> {
    let _workspace_reservation = reserve_tree_similarity_workspaces(1);
    let mut workspace = TreeSimilarityWorkspace::default();
    tree_similarity_with_workspace(left, right, &mut workspace)
}

pub(crate) fn tree_similarity_with_workspace(
    left: &StructuralTree,
    right: &StructuralTree,
    workspace: &mut TreeSimilarityWorkspace,
) -> Result<f64, TreeSimilarityError> {
    tree_similarity_with_workspace_and_budget(left, right, workspace, None)
}

pub(crate) fn tree_similarity_with_workspace_and_budget(
    left: &StructuralTree,
    right: &StructuralTree,
    workspace: &mut TreeSimilarityWorkspace,
    operation_budget: Option<&TreeSimilarityOperationBudget>,
) -> Result<f64, TreeSimilarityError> {
    if left == right {
        return Ok(1.0);
    }
    let denominator = left.node_count().max(right.node_count()) as f64;
    let distance =
        tree_edit_distance_with_workspace_and_budget(left, right, workspace, operation_budget)?;
    Ok((1.0 - distance / denominator).max(0.0))
}

#[cfg(test)]
fn tree_edit_distance(left: &StructuralTree, right: &StructuralTree) -> f64 {
    let mut workspace = TreeSimilarityWorkspace::default();
    tree_edit_distance_with_workspace(left, right, &mut workspace)
        .expect("test tree edit-distance workspace should be allocatable")
}

#[cfg(test)]
fn tree_edit_distance_with_workspace(
    left: &StructuralTree,
    right: &StructuralTree,
    workspace: &mut TreeSimilarityWorkspace,
) -> Result<f64, TreeSimilarityError> {
    tree_edit_distance_with_workspace_and_budget(left, right, workspace, None)
}

fn tree_edit_distance_with_workspace_and_budget(
    left: &StructuralTree,
    right: &StructuralTree,
    workspace: &mut TreeSimilarityWorkspace,
    shared_operation_budget: Option<&TreeSimilarityOperationBudget>,
) -> Result<f64, TreeSimilarityError> {
    tree_edit_distance_scaled_with_workspace(left, right, workspace, shared_operation_budget)
        .map(|distance| distance as f64 / EDIT_COST_SCALE as f64)
}

fn tree_edit_distance_scaled_with_workspace(
    left: &StructuralTree,
    right: &StructuralTree,
    workspace: &mut TreeSimilarityWorkspace,
    shared_operation_budget: Option<&TreeSimilarityOperationBudget>,
) -> Result<usize, TreeSimilarityError> {
    let left_len = left.labels.len();
    let right_len = right.labels.len();
    let (width, len, bytes) = distance_matrix_dimensions(left_len, right_len)?;
    workspace.try_reset(len, bytes)?;
    let mut local_operation_budget = TreeEditOperationBudget::new(MAX_TREE_EDIT_OPERATIONS);
    let mut operation_budgets = TreeEditOperationBudgets {
        local: &mut local_operation_budget,
        shared: shared_operation_budget,
    };

    for &left_root in &left.keyroots {
        for &right_root in &right.keyroots {
            forest_distance(
                left,
                right,
                &mut workspace.tree_distances,
                &mut workspace.forest_distances,
                width,
                ForestRoots {
                    left: left_root,
                    right: right_root,
                },
                &mut operation_budgets,
            )?;
        }
    }

    Ok(workspace.tree_distances[index(left_len, right_len, width)])
}

fn distance_matrix_dimensions(
    left_len: usize,
    right_len: usize,
) -> Result<(usize, usize, usize), TreeSimilarityError> {
    let exceeded = || TreeSimilarityError::MatrixTooLarge {
        cells: usize::MAX,
        bytes: usize::MAX,
    };
    let height = left_len.checked_add(1).ok_or_else(exceeded)?;
    let width = right_len.checked_add(1).ok_or_else(exceeded)?;
    let len = height.checked_mul(width).ok_or_else(exceeded)?;
    let bytes = len
        .checked_mul(std::mem::size_of::<usize>())
        .and_then(|one_matrix| one_matrix.checked_mul(2))
        .ok_or_else(exceeded)?;
    if len > MAX_DISTANCE_MATRIX_CELLS || bytes > MAX_TREE_SIMILARITY_WORKSPACE_BYTES {
        return Err(TreeSimilarityError::MatrixTooLarge { cells: len, bytes });
    }
    Ok((width, len, bytes))
}

fn forest_distance(
    left: &StructuralTree,
    right: &StructuralTree,
    tree_distances: &mut [usize],
    forest_distances: &mut [usize],
    width: usize,
    roots: ForestRoots,
    operation_budgets: &mut TreeEditOperationBudgets<'_>,
) -> Result<(), TreeSimilarityError> {
    let left_root = roots.left;
    let right_root = roots.right;
    let left_start = left.leftmost[left_root - 1];
    let right_start = right.leftmost[right_root - 1];
    let row_count = left_root - left_start + 2;
    let column_count = right_root - right_start + 2;

    for row in 1..row_count {
        forest_distances[index(row, 0, width)] =
            forest_distances[index(row - 1, 0, width)] + EDIT_COST_SCALE;
    }
    for column in 1..column_count {
        forest_distances[index(0, column, width)] =
            forest_distances[index(0, column - 1, width)] + EDIT_COST_SCALE;
    }

    for row in 1..row_count {
        let left_node = left_start + row - 1;
        for column in 1..column_count {
            operation_budgets.consume()?;
            let right_node = right_start + column - 1;
            let delete = forest_distances[index(row - 1, column, width)] + EDIT_COST_SCALE;
            let insert = forest_distances[index(row, column - 1, width)] + EDIT_COST_SCALE;

            if left.leftmost[left_node - 1] == left_start
                && right.leftmost[right_node - 1] == right_start
            {
                let rename = forest_distances[index(row - 1, column - 1, width)]
                    + rename_cost_scaled(left, left_node - 1, right, right_node - 1);
                let distance = delete.min(insert).min(rename);
                forest_distances[index(row, column, width)] = distance;
                tree_distances[index(left_node, right_node, width)] = distance;
            } else {
                let left_prefix = left.leftmost[left_node - 1] - left_start;
                let right_prefix = right.leftmost[right_node - 1] - right_start;
                let replace = forest_distances[index(left_prefix, right_prefix, width)]
                    + tree_distances[index(left_node, right_node, width)];
                forest_distances[index(row, column, width)] = delete.min(insert).min(replace);
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct ForestRoots {
    left: usize,
    right: usize,
}

struct TreeEditOperationBudget {
    operations: usize,
    limit: usize,
}

struct TreeEditOperationBudgets<'a> {
    local: &'a mut TreeEditOperationBudget,
    shared: Option<&'a TreeSimilarityOperationBudget>,
}

impl TreeEditOperationBudgets<'_> {
    #[inline]
    fn consume(&mut self) -> Result<(), TreeSimilarityError> {
        if let Some(shared) = self.shared {
            shared.consume()?;
        }
        self.local.consume()
    }
}

impl TreeEditOperationBudget {
    fn new(limit: usize) -> Self {
        Self {
            operations: 0,
            limit,
        }
    }

    #[inline]
    fn consume(&mut self) -> Result<(), TreeSimilarityError> {
        let next =
            self.operations
                .checked_add(1)
                .ok_or(TreeSimilarityError::OperationBudgetExceeded {
                    operations: usize::MAX,
                    limit: self.limit,
                })?;
        if next > self.limit {
            return Err(TreeSimilarityError::OperationBudgetExceeded {
                operations: next,
                limit: self.limit,
            });
        }
        self.operations = next;
        Ok(())
    }
}

#[inline]
fn index(row: usize, column: usize, width: usize) -> usize {
    row * width + column
}

fn rename_cost_scaled(
    left: &StructuralTree,
    left_node: usize,
    right: &StructuralTree,
    right_node: usize,
) -> usize {
    // Hash check first: unequal labels (the common case in this hot loop)
    // are rejected without comparing atom text; the full comparison then
    // guards against hash collisions.
    if left.label_hashes[left_node] == right.label_hashes[right_node]
        && left.labels[left_node] == right.labels[right_node]
    {
        0
    } else if matches!(
        (&left.labels[left_node], &right.labels[right_node]),
        (NodeLabel::Atom(_, _), NodeLabel::Atom(_, _))
    ) {
        ATOM_RENAME_COST
    } else {
        EDIT_COST_SCALE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::sexpr::{ByteOffset, ByteSpan, Path, SyntaxTree};

    fn form(input: &str) -> StructuralTree {
        let tree = SyntaxTree::parse(input).unwrap();
        StructuralTree::from_view(&tree.select_path(&Path::root_child(0)).unwrap().view())
    }

    fn assert_similarity_contract(left: &StructuralTree, right: &StructuralTree) -> f64 {
        let forward = tree_similarity(left, right).unwrap();
        let backward = tree_similarity(right, left).unwrap();

        assert!(forward.is_finite());
        assert!((0.0..=1.0).contains(&forward));
        assert!((forward - backward).abs() < f64::EPSILON);
        forward
    }

    #[test]
    fn workspace_limiter_blocks_until_a_reservation_is_released() {
        let limiter = TreeSimilarityWorkspaceLimiter::new(2);
        let reservation = limiter.acquire(2);
        assert_eq!(reservation.count(), 2);

        std::thread::scope(|scope| {
            let (attempting_tx, attempting_rx) = std::sync::mpsc::channel();
            let (acquired_tx, acquired_rx) = std::sync::mpsc::channel();
            let limiter = &limiter;
            scope.spawn(move || {
                attempting_tx.send(()).unwrap();
                let reservation = limiter.acquire(1);
                acquired_tx.send(reservation.count()).unwrap();
            });

            attempting_rx.recv().unwrap();
            assert_eq!(
                acquired_rx.recv_timeout(std::time::Duration::from_millis(50)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            );
            drop(reservation);
            assert_eq!(
                acquired_rx
                    .recv_timeout(std::time::Duration::from_secs(1))
                    .unwrap(),
                1
            );
        });
    }

    #[test]
    fn tree_edit_operation_counter_fails_closed_at_budget() {
        let mut budget = TreeEditOperationBudget {
            operations: MAX_TREE_EDIT_OPERATIONS,
            limit: MAX_TREE_EDIT_OPERATIONS,
        };

        assert_eq!(
            budget.consume(),
            Err(TreeSimilarityError::OperationBudgetExceeded {
                operations: MAX_TREE_EDIT_OPERATIONS + 1,
                limit: MAX_TREE_EDIT_OPERATIONS,
            })
        );
        assert_eq!(budget.operations, MAX_TREE_EDIT_OPERATIONS);
    }

    #[test]
    fn shared_operation_budget_is_exact_for_equal_label_multisets() {
        let left = form("(foo (bar a) b)");
        let right = form("(foo bar (a b))");
        assert_ne!(left, right);
        assert_eq!(left.sorted_label_hashes, right.sorted_label_hashes);
        assert_eq!(similarity_upper_bound(&left, &right), 1.0);

        let budget = TreeSimilarityOperationBudget::new(1);
        let mut workspace = TreeSimilarityWorkspace::default();
        assert_eq!(
            tree_similarity_with_workspace_and_budget(&left, &right, &mut workspace, Some(&budget),),
            Err(TreeSimilarityError::OperationBudgetExceeded {
                operations: 2,
                limit: 1,
            })
        );
        assert_eq!(budget.operations(), 1);
        assert!(budget.exhausted());

        assert!(tree_similarity(&left, &right).is_ok());
    }

    #[test]
    fn alpha_rename_is_highly_similar() {
        assert!(
            tree_similarity(
                &form("(let ((x 1)) (+ x 2))"),
                &form("(let ((y 1)) (+ y 2))")
            )
            .unwrap()
                > 0.9
        );
    }

    #[test]
    fn structural_difference_lowers_similarity() {
        let renamed = tree_similarity(&form("(foo a b)"), &form("(foo x y)")).unwrap();
        let changed = tree_similarity(&form("(foo a b)"), &form("(foo (bar a) b c)")).unwrap();
        assert!(changed < renamed);
    }

    #[test]
    fn adding_or_removing_one_wrapper_costs_one_edit() {
        let plain = form("(foo a)");
        let wrapped = form("((foo a))");

        assert!((tree_edit_distance(&plain, &wrapped) - 1.0).abs() < f64::EPSILON);
        assert!((tree_edit_distance(&wrapped, &plain) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn similarity_is_symmetric_and_bounded() {
        let left = form("'(foo a)");
        let right = form("(bar (a))");
        assert_similarity_contract(&left, &right);
    }

    #[test]
    fn identical_trees_have_maximum_similarity() {
        for input in ["atom", "()", "'(foo [bar] {baz})"] {
            let tree = form(input);
            assert_eq!(assert_similarity_contract(&tree, &tree), 1.0);
        }
    }

    #[test]
    fn reader_prefixes_are_structurally_significant() {
        let variants = [
            form("'value"),
            form("`value"),
            form(",value"),
            form(",@value"),
            form("#'value"),
        ];

        for left in 0..variants.len() {
            for right in (left + 1)..variants.len() {
                assert!(assert_similarity_contract(&variants[left], &variants[right]) < 1.0);
            }
        }
    }

    #[test]
    fn delimiters_are_structurally_significant() {
        let round = form("(value)");
        let square = form("[value]");
        let curly = form("{value}");

        assert!(assert_similarity_contract(&round, &square) < 1.0);
        assert!(assert_similarity_contract(&round, &curly) < 1.0);
        assert!(assert_similarity_contract(&square, &curly) < 1.0);
    }

    #[test]
    fn atom_list_and_empty_list_shapes_are_distinct() {
        let atom = form("value");
        let empty = form("()");
        let populated = form("(value)");

        assert!(assert_similarity_contract(&atom, &empty) < 1.0);
        assert!(assert_similarity_contract(&atom, &populated) < 1.0);
        assert!(assert_similarity_contract(&empty, &populated) < 1.0);
    }

    #[test]
    fn deep_and_wide_trees_preserve_similarity_contracts() {
        let deep_left = form(&format!("{}value{}", "(".repeat(64), ")".repeat(64)));
        let deep_right = form(&format!("{}other{}", "(".repeat(64), ")".repeat(64)));
        let wide_left = form(&format!("({})", vec!["value"; 128].join(" ")));
        let wide_right = form(&format!("({})", vec!["other"; 128].join(" ")));

        let deep_similarity = assert_similarity_contract(&deep_left, &deep_right);
        let wide_similarity = assert_similarity_contract(&wide_left, &wide_right);
        assert!(deep_similarity < 1.0);
        assert!(wide_similarity < 1.0);
    }

    #[test]
    fn structural_tree_conversion_preserves_postorder_metadata() {
        let tree = form("(root (left leaf) right)");

        assert_eq!(tree.labels.len(), 6);
        assert_eq!(tree.leftmost, vec![1, 2, 3, 2, 5, 1]);
        assert_eq!(tree.keyroots, vec![3, 4, 5, 6]);
    }

    #[test]
    fn structural_tree_conversion_handles_deep_views_iteratively() {
        let span = ByteSpan::new(ByteOffset::new(0), ByteOffset::new(0));
        let mut view = ExpressionView {
            kind: ExpressionKind::Atom,
            delimiter: None,
            reader_prefixes: Vec::new(),
            span,
            content_span: span,
            text: Some("leaf".to_string()),
            children: Vec::new(),
            symbol_offset: 0,
        };
        for _ in 0..10_000 {
            view = ExpressionView {
                kind: ExpressionKind::List,
                delimiter: Some(Delimiter::Paren),
                reader_prefixes: Vec::new(),
                span,
                content_span: span,
                text: None,
                children: vec![view],
                symbol_offset: 0,
            };
        }

        let structural = StructuralTree::from_view(&view);
        assert_eq!(structural.node_count(), 10_001);
        assert!(structural.leftmost.iter().all(|&leaf| leaf == 1));

        // This test targets conversion. Dropping a deeply owned ExpressionView
        // is independently recursive in Vec's destructor.
        std::mem::forget(view);
    }

    #[test]
    fn distance_matrix_dimensions_reject_overflow_without_allocating() {
        assert_eq!(
            distance_matrix_dimensions(2, 3),
            Ok((4, 12, 12 * std::mem::size_of::<usize>() * 2))
        );
        assert!(distance_matrix_dimensions(usize::MAX, 0).is_err());
        assert!(distance_matrix_dimensions(0, usize::MAX).is_err());
        assert!(distance_matrix_dimensions(usize::MAX / 2, 2).is_err());
    }

    #[test]
    fn distance_matrix_dimensions_enforce_cell_and_byte_budgets() {
        let error = distance_matrix_dimensions(MAX_DISTANCE_MATRIX_CELLS, 1).unwrap_err();
        assert!(matches!(error, TreeSimilarityError::MatrixTooLarge { .. }));
    }

    #[test]
    fn failed_workspace_growth_preserves_existing_buffers() {
        let mut workspace = TreeSimilarityWorkspace::default();
        workspace.try_reset(4, 64).unwrap();
        let tree_capacity = workspace.tree_distances.capacity();
        let forest_capacity = workspace.forest_distances.capacity();

        assert!(workspace.try_reset(usize::MAX, usize::MAX).is_err());
        assert_eq!(workspace.tree_distances.capacity(), tree_capacity);
        assert_eq!(workspace.forest_distances.capacity(), forest_capacity);
    }

    #[test]
    fn workspace_reservation_failure_is_reported_without_panicking() {
        let mut workspace = TreeSimilarityWorkspace::default();
        assert!(workspace.try_reset(usize::MAX, usize::MAX).is_err());
        assert!(workspace.tree_distances.is_empty());
        assert!(workspace.forest_distances.is_empty());
    }
}
