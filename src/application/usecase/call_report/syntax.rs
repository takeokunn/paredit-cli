use crate::domain::definition::{DefinitionCategory, definition_name_child_index};
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

pub(super) fn definition_body_start_index(
    view: &ExpressionView,
    head: &str,
    category: Option<DefinitionCategory>,
) -> usize {
    match (category, lambda_list_index(view, head)) {
        (Some(category), Some(index)) if category.is_callable() => index + 1,
        (Some(category), None) if category.is_callable() => 3,
        (Some(_), _) => 2,
        (None, _) => 0,
    }
}

pub(super) fn definition_name<'a>(view: &'a ExpressionView, head: &str) -> Option<&'a str> {
    definition_name_child_index(head).and_then(|index| atom_child(view, index))
}

pub(super) fn list_head(view: &ExpressionView) -> Option<&str> {
    atom_child(view, 0)
}

fn lambda_list_index(view: &ExpressionView, head: &str) -> Option<usize> {
    match head {
        "defun"
        | "cl-defun"
        | "defsubst"
        | "definline"
        | "defmacro"
        | "cl-defmacro"
        | "define-compiler-macro"
        | "define-modify-macro"
        | "define-setf-expander"
        | "defsetf"
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

fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
