use anyhow::Result;

use crate::domain::definition::{classify_definition_head, definition_name_child_index};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path, SyntaxTree};

use super::syntax::{atom_child, count_lambda_parameters, list_child, list_head};
use super::types::CallGraphDefinitionItem;

pub(super) fn collect_call_graph_definitions(
    tree: &SyntaxTree,
    dialect: Dialect,
) -> Result<Vec<CallGraphDefinitionItem>> {
    let mut items = Vec::new();

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_call_graph_definitions_from_view(&view, dialect, path_indexes, &mut items);
    }

    Ok(items)
}

fn collect_call_graph_definitions_from_view(
    view: &ExpressionView,
    dialect: Dialect,
    path_indexes: Vec<usize>,
    items: &mut Vec<CallGraphDefinitionItem>,
) {
    if view.kind == crate::domain::sexpr::ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && let Some(head) = list_head(view)
        && let Some(category) = classify_definition_head(dialect, head)
    {
        let name = definition_name(view, head).map(str::to_string);
        let parameter_count = lambda_list_index(view, head)
            .and_then(|index| list_child(view, index))
            .map(count_lambda_parameters)
            .unwrap_or(0);

        items.push(CallGraphDefinitionItem {
            name,
            category,
            path: path_indexes.clone(),
            span: view.span,
            parameter_count,
        });
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_call_graph_definitions_from_view(child, dialect, child_path, items);
    }
}

fn definition_name<'a>(view: &'a ExpressionView, head: &str) -> Option<&'a str> {
    definition_name_child_index(head).and_then(|index| atom_child(view, index))
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
        | "define"
        | "lambda"
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
