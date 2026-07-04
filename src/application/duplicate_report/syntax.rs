use crate::domain::sexpr::{ExpressionKind, ExpressionView};

pub(super) fn expression_node_count(view: &ExpressionView) -> usize {
    1 + view
        .children
        .iter()
        .map(expression_node_count)
        .sum::<usize>()
}

pub(super) fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .and_then(|text| text)
        .filter(|text| !text.is_empty())
}
