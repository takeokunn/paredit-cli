use crate::domain::sexpr::{ExpressionKind, ExpressionView};

pub(super) fn list_head(view: &ExpressionView) -> Option<&str> {
    atom_child(view, 0)
}

pub(super) fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
