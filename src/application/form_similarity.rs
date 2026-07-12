use std::cmp::Ordering;

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

#[derive(Default)]
pub(crate) struct TreeSimilarityWorkspace {
    tree_distances: Vec<usize>,
    forest_distances: Vec<usize>,
}

impl TreeSimilarityWorkspace {
    fn reset(&mut self, len: usize) {
        self.tree_distances.resize(len, 0);
        self.tree_distances[..len].fill(0);
        self.forest_distances.resize(len, 0);
        self.forest_distances[..len].fill(0);
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

        fn visit(
            view: &ExpressionView,
            labels: &mut Vec<NodeLabel>,
            leftmost: &mut Vec<usize>,
        ) -> (usize, usize) {
            let mut node_count = 1;
            let mut first_leaf = None;

            for child_view in &view.children {
                let (child_count, child_leaf) = visit(child_view, labels, leftmost);
                node_count += child_count;
                if first_leaf.is_none() {
                    first_leaf = Some(child_leaf);
                }
            }

            labels.push(label(view));
            let index = labels.len();
            let leaf = first_leaf.unwrap_or(index);
            leftmost.push(leaf);

            (node_count, leaf)
        }

        let mut labels = Vec::new();
        let mut leftmost = Vec::new();
        let (node_count, _) = visit(view, &mut labels, &mut leftmost);
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
    value.to_le_bytes().iter().fold(hash, |hash, &byte| fnv_byte(hash, byte))
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
    let shared =
        sorted_intersection_count(&left.sorted_label_hashes, &right.sorted_label_hashes);
    let lower_bound_scaled = EDIT_COST_SCALE * (left_count + right_count - 2 * matched)
        + ATOM_RENAME_COST * matched.saturating_sub(shared);
    1.0 - lower_bound_scaled as f64 / (EDIT_COST_SCALE * left_count.max(right_count)) as f64
}

pub fn tree_similarity(left: &StructuralTree, right: &StructuralTree) -> f64 {
    let mut workspace = TreeSimilarityWorkspace::default();
    tree_similarity_with_workspace(left, right, &mut workspace)
}

pub(crate) fn tree_similarity_with_workspace(
    left: &StructuralTree,
    right: &StructuralTree,
    workspace: &mut TreeSimilarityWorkspace,
) -> f64 {
    if left == right {
        return 1.0;
    }
    let denominator = left.node_count().max(right.node_count()) as f64;
    (1.0 - tree_edit_distance_with_workspace(left, right, workspace) / denominator).max(0.0)
}

#[cfg(test)]
fn tree_edit_distance(left: &StructuralTree, right: &StructuralTree) -> f64 {
    let mut workspace = TreeSimilarityWorkspace::default();
    tree_edit_distance_with_workspace(left, right, &mut workspace)
}

fn tree_edit_distance_with_workspace(
    left: &StructuralTree,
    right: &StructuralTree,
    workspace: &mut TreeSimilarityWorkspace,
) -> f64 {
    tree_edit_distance_scaled_with_workspace(left, right, workspace) as f64 / EDIT_COST_SCALE as f64
}

fn tree_edit_distance_scaled_with_workspace(
    left: &StructuralTree,
    right: &StructuralTree,
    workspace: &mut TreeSimilarityWorkspace,
) -> usize {
    let left_len = left.labels.len();
    let right_len = right.labels.len();
    let width = right_len + 1;
    let len = (left_len + 1) * width;
    workspace.reset(len);

    for &left_root in &left.keyroots {
        for &right_root in &right.keyroots {
            forest_distance(
                left,
                right,
                &mut workspace.tree_distances,
                &mut workspace.forest_distances,
                width,
                left_root,
                right_root,
            );
        }
    }

    workspace.tree_distances[index(left_len, right_len, width)]
}

fn forest_distance(
    left: &StructuralTree,
    right: &StructuralTree,
    tree_distances: &mut [usize],
    forest_distances: &mut [usize],
    width: usize,
    left_root: usize,
    right_root: usize,
) {
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
    use crate::domain::sexpr::{Path, SyntaxTree};

    fn form(input: &str) -> StructuralTree {
        let tree = SyntaxTree::parse(input).unwrap();
        StructuralTree::from_view(&tree.select_path(&Path::root_child(0)).unwrap().view())
    }

    fn assert_similarity_contract(left: &StructuralTree, right: &StructuralTree) -> f64 {
        let forward = tree_similarity(left, right);
        let backward = tree_similarity(right, left);

        assert!(forward.is_finite());
        assert!((0.0..=1.0).contains(&forward));
        assert!((forward - backward).abs() < f64::EPSILON);
        forward
    }

    #[test]
    fn alpha_rename_is_highly_similar() {
        assert!(
            tree_similarity(
                &form("(let ((x 1)) (+ x 2))"),
                &form("(let ((y 1)) (+ y 2))")
            ) > 0.9
        );
    }

    #[test]
    fn structural_difference_lowers_similarity() {
        let renamed = tree_similarity(&form("(foo a b)"), &form("(foo x y)"));
        let changed = tree_similarity(&form("(foo a b)"), &form("(foo (bar a) b c)"));
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
}
