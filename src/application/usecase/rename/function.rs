use anyhow::Result;

use crate::domain::definition::{classify_definition_head, definition_name_child_index};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

use super::RenameFunctionOccurrence;
use super::selection::{definition_name, list_head};

pub fn collect_callable_definition_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<RenameFunctionOccurrence>> {
    let mut renames = Vec::new();

    for (top_index, _) in tree.root_children().iter().enumerate() {
        let form_path = Path::from_indexes(vec![top_index]);
        let view = tree.select_path(&form_path)?.view();
        let Some(head) = list_head(&view) else {
            continue;
        };
        if classify_definition_head(dialect, head).is_none() {
            continue;
        }
        if definition_name(&view, head) != Some(from.as_str()) {
            continue;
        }
        let Some(name_index) = definition_name_child_index(head) else {
            continue;
        };
        let name_path = Path::from_indexes(vec![top_index, name_index]);
        let name_view = tree.select_path(&name_path)?.view();
        renames.push(RenameFunctionOccurrence {
            path: name_path.to_string(),
            span: name_view.span,
            text: from.as_str().to_owned(),
            replacement: to.as_str().to_owned(),
        });
    }

    Ok(renames)
}

pub fn collect_function_call_head_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<RenameFunctionOccurrence>> {
    let mut renames = Vec::new();
    for (index, _) in tree.root_children().iter().enumerate() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_function_call_head_renames_from_view(
            &view,
            dialect,
            path_indexes,
            from,
            to,
            &mut renames,
        );
    }
    Ok(renames)
}

fn collect_function_call_head_renames_from_view(
    view: &ExpressionView,
    dialect: Dialect,
    path_indexes: Vec<usize>,
    from: &SymbolName,
    to: &SymbolName,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let mut first_callable_child_index = 0;

    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        if let Some(head) = list_head(view) {
            let category = classify_definition_head(dialect, head);
            if head == from.as_str() && classify_definition_head(dialect, head).is_none() {
                if let Some(head_view) = view.children.first() {
                    let mut head_path = path_indexes.clone();
                    head_path.push(0);
                    renames.push(RenameFunctionOccurrence {
                        path: Path::from_indexes(head_path).to_string(),
                        span: head_view.span,
                        text: head.to_owned(),
                        replacement: to.as_str().to_owned(),
                    });
                }
            }
            if category.is_some() {
                first_callable_child_index = definition_body_start_index(category);
            }
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        if index < first_callable_child_index {
            continue;
        }
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_function_call_head_renames_from_view(child, dialect, child_path, from, to, renames);
    }
}

fn definition_body_start_index(
    category: Option<crate::domain::definition::DefinitionCategory>,
) -> usize {
    match category {
        Some(category) if category.is_callable() => 3,
        Some(_) => 2,
        None => 0,
    }
}
