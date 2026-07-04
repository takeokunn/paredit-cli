use anyhow::Result;

use crate::application::usecase::call_report::syntax::{
    definition_body_start_index, definition_name, list_head,
};
use crate::application::usecase::call_report::types::CallReportItem;
use crate::application::usecase::callable_scope::{
    LocalCallableForm, common_lisp_local_callable_form, is_local_callable_bound,
    local_callable_names,
};
use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

pub fn build_call_report(
    tree: &SyntaxTree,
    dialect: Dialect,
    symbol: Option<&SymbolName>,
    include_definitions: bool,
) -> Result<Vec<CallReportItem>> {
    let mut calls = Vec::new();
    let ctx = CallReportTraversal {
        dialect,
        symbol,
        include_definitions,
    };

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_call_report_items_from_view(&view, path_indexes, None, &[], &ctx, &mut calls);
    }

    calls.sort_by_key(|call| call.span.start());
    Ok(calls)
}

struct CallReportTraversal<'a> {
    dialect: Dialect,
    symbol: Option<&'a SymbolName>,
    include_definitions: bool,
}

fn collect_call_report_items_from_view(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    enclosing_definition: Option<String>,
    local_callables: &[String],
    ctx: &CallReportTraversal<'_>,
    calls: &mut Vec<CallReportItem>,
) {
    let mut child_enclosing_definition = enclosing_definition.clone();
    let mut first_callable_child_index = 0;

    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && let Some(head) = list_head(view)
    {
        if let Some(form) = common_lisp_local_callable_form(ctx.dialect, head) {
            collect_local_callable_form_calls(
                view,
                path_indexes,
                enclosing_definition,
                local_callables,
                form,
                ctx,
                calls,
            );
            return;
        }

        let category = classify_definition_head(ctx.dialect, head);
        let matches_symbol = ctx.symbol.is_none_or(|target| head == target.as_str());

        if matches_symbol
            && (ctx.include_definitions || category.is_none())
            && !is_local_callable_bound(local_callables, head)
        {
            calls.push(CallReportItem {
                path: Path::from_indexes(path_indexes.clone()).to_string(),
                span: view.span,
                head: head.to_owned(),
                argument_count: view.children.len().saturating_sub(1),
                category,
                enclosing_definition: enclosing_definition.clone(),
            });
        }

        if category.is_some() {
            child_enclosing_definition = definition_name(view, head).map(ToOwned::to_owned);
            first_callable_child_index = definition_body_start_index(category);
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        if index < first_callable_child_index {
            continue;
        }
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_call_report_items_from_view(
            child,
            child_path,
            child_enclosing_definition.clone(),
            local_callables,
            ctx,
            calls,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_local_callable_form_calls(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    enclosing_definition: Option<String>,
    local_callables: &[String],
    form: LocalCallableForm,
    ctx: &CallReportTraversal<'_>,
    calls: &mut Vec<CallReportItem>,
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
                collect_call_report_items_from_view(
                    child,
                    child_path,
                    enclosing_definition.clone(),
                    binding_body_scope,
                    ctx,
                    calls,
                );
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_call_report_items_from_view(
            child,
            child_path,
            enclosing_definition.clone(),
            &body_scope,
            ctx,
            calls,
        );
    }
}
