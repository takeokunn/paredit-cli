use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, ReaderPrefix};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralTree {
    labels: Vec<NodeLabel>,
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

        (
            Self {
                labels,
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
                    + rename_cost_scaled(
                        &left.labels[left_node - 1],
                        &right.labels[right_node - 1],
                    );
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

fn rename_cost_scaled(left: &NodeLabel, right: &NodeLabel) -> usize {
    if left == right {
        0
    } else if matches!(
        (left, right),
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
