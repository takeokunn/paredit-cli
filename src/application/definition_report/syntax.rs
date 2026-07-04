use crate::domain::definition::definition_name_child_index;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

pub fn definition_name<'a>(view: &'a ExpressionView, head: &str) -> Option<&'a str> {
    definition_name_child_index(head).and_then(|index| atom_child(view, index))
}

pub fn lambda_list_index(view: &ExpressionView, head: &str) -> Option<usize> {
    match head {
        "defun"
        | "cl-defun"
        | "defsubst"
        | "definline"
        | "defmacro"
        | "cl-defmacro"
        | "define-compiler-macro"
        | "defn"
        | "defn-" => list_child_index(view, 2),
        "defmethod" | "cl-defmethod" => (2..view.children.len()).find(|&index| {
            matches!(
                view.children[index].delimiter,
                Some(Delimiter::Paren | Delimiter::Bracket)
            )
        }),
        "defgeneric" | "cl-defgeneric" => list_child_index(view, 2),
        "deftest" | "ert-deftest" | "define-test" | "define-ert-test" => list_child_index(view, 2),
        _ => None,
    }
}

fn list_child_index(view: &ExpressionView, index: usize) -> Option<usize> {
    view.children
        .get(index)
        .and_then(|child| (child.kind == ExpressionKind::List).then_some(index))
}

pub fn body_form_count(view: &ExpressionView, lambda_index: Option<usize>) -> Option<usize> {
    lambda_index
        .map(|index| view.children.len().saturating_sub(index + 1))
        .or_else(|| (view.children.len() >= 2).then_some(view.children.len().saturating_sub(2)))
}

pub fn count_lambda_parameters(lambda_list: &ExpressionView) -> usize {
    lambda_list
        .children
        .iter()
        .filter(|child| match child.kind {
            ExpressionKind::Atom => atom_text(child).is_some_and(|text| !text.starts_with('&')),
            ExpressionKind::List => true,
            ExpressionKind::Root => false,
        })
        .count()
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

pub(super) fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

pub(super) fn list_head(view: &ExpressionView) -> Option<&str> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return None;
    }

    atom_child(view, 0)
}
