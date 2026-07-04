use crate::domain::definition::definition_name_child_index;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

pub(super) fn list_head(view: &ExpressionView) -> Option<&str> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return None;
    }
    atom_child(view, 0)
}

pub(super) fn definition_name<'a>(view: &'a ExpressionView, head: &str) -> Option<&'a str> {
    definition_name_child_index(head).and_then(|index| atom_child(view, index))
}

fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
