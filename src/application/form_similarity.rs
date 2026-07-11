use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, ReaderPrefix};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralTree {
    label: NodeLabel,
    children: Vec<StructuralTree>,
    node_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeLabel {
    Root(Vec<ReaderPrefix>),
    List(Option<Delimiter>, Vec<ReaderPrefix>),
    Atom(String, Vec<ReaderPrefix>),
}

impl StructuralTree {
    pub fn from_view(view: &ExpressionView) -> Self {
        let children = view
            .children
            .iter()
            .map(Self::from_view)
            .collect::<Vec<_>>();
        let node_count = 1 + children.iter().map(|child| child.node_count).sum::<usize>();
        let label = match view.kind {
            ExpressionKind::Root => NodeLabel::Root(view.reader_prefixes.clone()),
            ExpressionKind::List => NodeLabel::List(view.delimiter, view.reader_prefixes.clone()),
            ExpressionKind::Atom => NodeLabel::Atom(
                view.text.clone().unwrap_or_default(),
                view.reader_prefixes.clone(),
            ),
        };
        Self {
            label,
            children,
            node_count,
        }
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }
}

pub fn tree_similarity(left: &StructuralTree, right: &StructuralTree) -> f64 {
    let denominator = left.node_count.max(right.node_count) as f64;
    (1.0 - tree_edit_distance(left, right) / denominator).max(0.0)
}

struct IndexedTree<'a> {
    labels: Vec<&'a NodeLabel>,
    leftmost: Vec<usize>,
    keyroots: Vec<usize>,
}

fn index_tree(tree: &StructuralTree) -> IndexedTree<'_> {
    fn visit<'a>(
        tree: &'a StructuralTree,
        labels: &mut Vec<&'a NodeLabel>,
        leftmost: &mut Vec<usize>,
    ) -> usize {
        let first_leaf = tree
            .children
            .first()
            .map(|child| visit(child, labels, leftmost));

        for child in tree.children.iter().skip(1) {
            visit(child, labels, leftmost);
        }

        labels.push(&tree.label);
        let index = labels.len();
        let leaf = first_leaf.unwrap_or(index);
        leftmost.push(leaf);
        leaf
    }

    let mut labels = Vec::with_capacity(tree.node_count);
    let mut leftmost = Vec::with_capacity(tree.node_count);
    visit(tree, &mut labels, &mut leftmost);

    let mut last_for_leaf = std::collections::BTreeMap::new();
    for (offset, leaf) in leftmost.iter().copied().enumerate() {
        last_for_leaf.insert(leaf, offset + 1);
    }
    let mut keyroots = last_for_leaf.into_values().collect::<Vec<_>>();
    keyroots.sort_unstable();

    IndexedTree {
        labels,
        leftmost,
        keyroots,
    }
}

fn tree_edit_distance(left: &StructuralTree, right: &StructuralTree) -> f64 {
    let left = index_tree(left);
    let right = index_tree(right);
    let mut tree_distances = vec![vec![0.0; right.labels.len() + 1]; left.labels.len() + 1];

    for &left_root in &left.keyroots {
        for &right_root in &right.keyroots {
            forest_distance(&left, &right, &mut tree_distances, left_root, right_root);
        }
    }

    tree_distances[left.labels.len()][right.labels.len()]
}

fn forest_distance(
    left: &IndexedTree<'_>,
    right: &IndexedTree<'_>,
    tree_distances: &mut [Vec<f64>],
    left_root: usize,
    right_root: usize,
) {
    let left_start = left.leftmost[left_root - 1];
    let right_start = right.leftmost[right_root - 1];
    let row_count = left_root - left_start + 2;
    let column_count = right_root - right_start + 2;
    let mut forest_distances = vec![vec![0.0; column_count]; row_count];

    for row in 1..row_count {
        forest_distances[row][0] = forest_distances[row - 1][0] + 1.0;
    }
    for column in 1..column_count {
        forest_distances[0][column] = forest_distances[0][column - 1] + 1.0;
    }

    for row in 1..row_count {
        let left_node = left_start + row - 1;
        for column in 1..column_count {
            let right_node = right_start + column - 1;
            let delete = forest_distances[row - 1][column] + 1.0;
            let insert = forest_distances[row][column - 1] + 1.0;

            if left.leftmost[left_node - 1] == left_start
                && right.leftmost[right_node - 1] == right_start
            {
                let rename = forest_distances[row - 1][column - 1]
                    + rename_cost(left.labels[left_node - 1], right.labels[right_node - 1]);
                let distance = delete.min(insert).min(rename);
                forest_distances[row][column] = distance;
                tree_distances[left_node][right_node] = distance;
            } else {
                let left_prefix = left.leftmost[left_node - 1] - left_start;
                let right_prefix = right.leftmost[right_node - 1] - right_start;
                let replace = forest_distances[left_prefix][right_prefix]
                    + tree_distances[left_node][right_node];
                forest_distances[row][column] = delete.min(insert).min(replace);
            }
        }
    }
}

fn rename_cost(left: &NodeLabel, right: &NodeLabel) -> f64 {
    if left == right {
        0.0
    } else if matches!(
        (left, right),
        (NodeLabel::Atom(_, _), NodeLabel::Atom(_, _))
    ) {
        0.3
    } else {
        1.0
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
        let forward = tree_similarity(&left, &right);
        let backward = tree_similarity(&right, &left);
        assert!((forward - backward).abs() < f64::EPSILON);
        assert!((0.0..=1.0).contains(&forward));
    }
}
