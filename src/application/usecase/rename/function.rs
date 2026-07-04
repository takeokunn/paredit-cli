use anyhow::Result;

use crate::application::usecase::callable_scope::{
    LocalCallableForm, common_lisp_local_callable_form, is_local_callable_bound,
    local_callable_names,
};
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
            &[],
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
    local_callables: &[String],
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let mut first_callable_child_index = 0;

    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        if let Some(head) = list_head(view) {
            if let Some(form) = common_lisp_local_callable_form(dialect, head) {
                collect_local_callable_function_call_renames(
                    view,
                    dialect,
                    path_indexes,
                    from,
                    to,
                    local_callables,
                    form,
                    renames,
                );
                return;
            }

            let category = classify_definition_head(dialect, head);
            if head == from.as_str()
                && classify_definition_head(dialect, head).is_none()
                && !is_local_callable_bound(local_callables, head)
            {
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
        collect_function_call_head_renames_from_view(
            child,
            dialect,
            child_path,
            from,
            to,
            local_callables,
            renames,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_local_callable_function_call_renames(
    view: &ExpressionView,
    dialect: Dialect,
    path_indexes: Vec<usize>,
    from: &SymbolName,
    to: &SymbolName,
    local_callables: &[String],
    form: LocalCallableForm,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let local_names = local_callable_names(view);
    let mut body_scope = local_callables.to_vec();
    body_scope.extend(local_names.iter().cloned());

    if let Some(bindings) = view.children.get(1) {
        let binding_body_scope = match form {
            LocalCallableForm::Labels => body_scope.as_slice(),
            LocalCallableForm::Flet
            | LocalCallableForm::Macrolet
            | LocalCallableForm::CompilerMacrolet => local_callables,
        };
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                let mut child_path = path_indexes.clone();
                child_path.extend([1, binding_index, child_index]);
                collect_function_call_head_renames_from_view(
                    child,
                    dialect,
                    child_path,
                    from,
                    to,
                    binding_body_scope,
                    renames,
                );
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_function_call_head_renames_from_view(
            child,
            dialect,
            child_path,
            from,
            to,
            &body_scope,
            renames,
        );
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
