use crate::domain::definition::{DefinitionCategory, definition_name_child_index};
use crate::domain::sexpr::{ExpressionKind, ExpressionView};

pub(super) fn definition_body_start_index(category: Option<DefinitionCategory>) -> usize {
    match category {
        Some(category) if category.is_callable() => 3,
        Some(_) => 2,
        None => 0,
    }
}

pub(super) fn definition_name<'a>(view: &'a ExpressionView, head: &str) -> Option<&'a str> {
    definition_name_child_index(head).and_then(|index| atom_child(view, index))
}

pub(super) fn list_head(view: &ExpressionView) -> Option<&str> {
    atom_child(view, 0)
}

fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
