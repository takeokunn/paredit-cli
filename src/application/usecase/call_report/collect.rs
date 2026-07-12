use anyhow::Result;

use crate::application::usecase::call_report::syntax::list_head;
use crate::application::usecase::call_report::types::CallReportItem;
use crate::application::usecase::callable_scope::{
    common_lisp_local_callable_form, is_local_callable_bound, local_callable_binding_body_scope,
    local_callable_body_scope,
};
use crate::domain::common_lisp::{
    CommonLispBindingListShape, CommonLispBindingRefactorForm, CommonLispLocalCallableForm,
    CommonLispOperator, CommonLispSlotBindingForm, common_lisp_symbol_reference_eq,
    is_common_lisp_declaration_form,
};
use crate::domain::definition::definition_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::apply_reader_prefix_context;
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
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collect_call_report_items_from_view(&view, path, None, &[], 0, &ctx, &mut calls);
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
    path: Path,
    enclosing_definition: Option<String>,
    local_callables: &[String],
    quasiquote_depth: usize,
    ctx: &CallReportTraversal<'_>,
    calls: &mut Vec<CallReportItem>,
) {
    let Some(quasiquote_depth) = apply_reader_prefix_context(view, quasiquote_depth) else {
        return;
    };

    if quasiquote_depth > 0 {
        for (index, child) in view.children.iter().enumerate() {
            let child_path = path.child(index);
            collect_call_report_items_from_view(
                child,
                child_path,
                enclosing_definition.clone(),
                local_callables,
                quasiquote_depth,
                ctx,
                calls,
            );
        }
        return;
    }

    let mut child_enclosing_definition = enclosing_definition.clone();
    let mut definition_body_range = None;

    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        if let Some(head) = list_head(view) {
            if let Some(form) = common_lisp_local_callable_form(ctx.dialect, head) {
                collect_local_callable_form_calls(
                    view,
                    path,
                    enclosing_definition,
                    local_callables,
                    quasiquote_depth,
                    form,
                    ctx,
                    calls,
                );
                return;
            }

            if is_common_lisp_declaration_form(head) {
                return;
            }

            if CommonLispOperator::from_head(head)
                .is_some_and(|operator| operator == CommonLispOperator::Locally)
            {
                collect_locally_call_report_items(
                    view,
                    path,
                    enclosing_definition,
                    local_callables,
                    quasiquote_depth,
                    ctx,
                    calls,
                );
                return;
            }

            if let Some(refactor_form) = ctx
                .dialect
                .common_lisp_binding_refactor_form_for_head(head)
                .filter(|form| form.binding_list_shape().is_some())
            {
                collect_binding_refactor_form_calls(
                    view,
                    path,
                    enclosing_definition,
                    local_callables,
                    quasiquote_depth,
                    refactor_form,
                    head,
                    ctx,
                    calls,
                );
                return;
            }

            let shape = definition_shape(ctx.dialect, view, head);
            let matches_symbol = ctx
                .symbol
                .is_none_or(|target| common_lisp_symbol_reference_eq(head, target.as_str()));

            if matches_symbol
                && (ctx.include_definitions || shape.is_none())
                && !is_local_callable_bound(local_callables, head)
            {
                calls.push(CallReportItem {
                    path: path.to_string(),
                    span: view.span,
                    head: head.to_owned(),
                    argument_count: view.children.len().saturating_sub(1),
                    category: shape.map(|shape| shape.category),
                    enclosing_definition: enclosing_definition.clone(),
                });
            }

            if let Some(shape) = shape {
                child_enclosing_definition = shape.name(view).map(ToOwned::to_owned);
                definition_body_range = Some(shape.body_range());
            }
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        if let Some(range) = definition_body_range {
            if range.child_path(&path, index).is_none() {
                continue;
            }
        }
        let child_path = path.child(index);
        collect_call_report_items_from_view(
            child,
            child_path,
            child_enclosing_definition.clone(),
            local_callables,
            quasiquote_depth,
            ctx,
            calls,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_binding_refactor_form_calls(
    view: &ExpressionView,
    path: Path,
    enclosing_definition: Option<String>,
    local_callables: &[String],
    quasiquote_depth: usize,
    refactor_form: CommonLispBindingRefactorForm,
    head: &str,
    ctx: &CallReportTraversal<'_>,
    calls: &mut Vec<CallReportItem>,
) {
    let shape = definition_shape(ctx.dialect, view, head);
    let matches_symbol = ctx
        .symbol
        .is_none_or(|target| common_lisp_symbol_reference_eq(head, target.as_str()));

    if matches_symbol
        && (ctx.include_definitions || shape.is_none())
        && !is_local_callable_bound(local_callables, head)
    {
        calls.push(CallReportItem {
            path: path.to_string(),
            span: view.span,
            head: head.to_owned(),
            argument_count: view.children.len().saturating_sub(1),
            category: shape.map(|shape| shape.category),
            enclosing_definition: enclosing_definition.clone(),
        });
    }

    if let Some(bindings) = view.children.get(1) {
        collect_binding_list_call_report_items(
            bindings,
            path.child(1),
            enclosing_definition.clone(),
            local_callables,
            quasiquote_depth,
            refactor_form,
            ctx,
            calls,
        );
    }

    for (index, child) in view
        .children
        .iter()
        .enumerate()
        .skip(refactor_form.remove_unused_body_start_index())
    {
        let child_path = path.child(index);
        collect_call_report_items_from_view(
            child,
            child_path,
            enclosing_definition.clone(),
            local_callables,
            quasiquote_depth,
            ctx,
            calls,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_binding_list_call_report_items(
    bindings: &ExpressionView,
    path: Path,
    enclosing_definition: Option<String>,
    local_callables: &[String],
    quasiquote_depth: usize,
    refactor_form: CommonLispBindingRefactorForm,
    ctx: &CallReportTraversal<'_>,
    calls: &mut Vec<CallReportItem>,
) {
    let Some(shape) = refactor_form.binding_list_shape() else {
        return;
    };

    match shape {
        CommonLispBindingListShape::NameValuePairs => {
            if bindings.kind == ExpressionKind::List
                && bindings.delimiter == Some(Delimiter::Bracket)
            {
                for (index, child) in bindings.children.iter().enumerate() {
                    if index % 2 == 0 {
                        continue;
                    }
                    collect_call_report_items_from_view(
                        child,
                        path.child(index),
                        enclosing_definition.clone(),
                        local_callables,
                        quasiquote_depth,
                        ctx,
                        calls,
                    );
                }
                return;
            }

            for (binding_index, binding) in bindings.children.iter().enumerate() {
                collect_binding_children_from_index(
                    binding,
                    path.child(binding_index),
                    1,
                    enclosing_definition.clone(),
                    local_callables,
                    quasiquote_depth,
                    ctx,
                    calls,
                );
            }
        }
        CommonLispBindingListShape::LocalCallableDefinitions(_) => {}
        CommonLispBindingListShape::VariableSpecs(_) => {
            for (binding_index, binding) in bindings.children.iter().enumerate() {
                if binding.kind == ExpressionKind::Atom {
                    continue;
                }
                collect_binding_children_from_index(
                    binding,
                    path.child(binding_index),
                    1,
                    enclosing_definition.clone(),
                    local_callables,
                    quasiquote_depth,
                    ctx,
                    calls,
                );
            }
        }
        CommonLispBindingListShape::SlotBindings(form) => {
            for (binding_index, binding) in bindings.children.iter().enumerate() {
                let start_index = match form {
                    CommonLispSlotBindingForm::WithSlots
                        if binding.kind == ExpressionKind::Atom =>
                    {
                        continue;
                    }
                    CommonLispSlotBindingForm::WithSlots
                    | CommonLispSlotBindingForm::WithAccessors => 1,
                };
                collect_binding_children_from_index(
                    binding,
                    path.child(binding_index),
                    start_index,
                    enclosing_definition.clone(),
                    local_callables,
                    quasiquote_depth,
                    ctx,
                    calls,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_binding_children_from_index(
    binding: &ExpressionView,
    path: Path,
    start_index: usize,
    enclosing_definition: Option<String>,
    local_callables: &[String],
    quasiquote_depth: usize,
    ctx: &CallReportTraversal<'_>,
    calls: &mut Vec<CallReportItem>,
) {
    for (child_index, child) in binding.children.iter().enumerate().skip(start_index) {
        collect_call_report_items_from_view(
            child,
            path.child(child_index),
            enclosing_definition.clone(),
            local_callables,
            quasiquote_depth,
            ctx,
            calls,
        );
    }
}

fn collect_locally_call_report_items(
    view: &ExpressionView,
    path: Path,
    enclosing_definition: Option<String>,
    local_callables: &[String],
    quasiquote_depth: usize,
    ctx: &CallReportTraversal<'_>,
    calls: &mut Vec<CallReportItem>,
) {
    for (index, child) in view.children.iter().enumerate().skip(2) {
        let child_path = path.child(index);
        collect_call_report_items_from_view(
            child,
            child_path,
            enclosing_definition.clone(),
            local_callables,
            quasiquote_depth,
            ctx,
            calls,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_local_callable_form_calls(
    view: &ExpressionView,
    path: Path,
    enclosing_definition: Option<String>,
    local_callables: &[String],
    quasiquote_depth: usize,
    form: CommonLispLocalCallableForm,
    ctx: &CallReportTraversal<'_>,
    calls: &mut Vec<CallReportItem>,
) {
    let body_scope = local_callable_body_scope(local_callables, view);

    if let Some(bindings) = view.children.get(1) {
        let binding_body_scope =
            local_callable_binding_body_scope(form, local_callables, &body_scope);
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                let child_path = path.descendant([1, binding_index, child_index]);
                collect_call_report_items_from_view(
                    child,
                    child_path,
                    enclosing_definition.clone(),
                    binding_body_scope,
                    quasiquote_depth,
                    ctx,
                    calls,
                );
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        let child_path = path.child(index);
        collect_call_report_items_from_view(
            child,
            child_path,
            enclosing_definition.clone(),
            &body_scope,
            quasiquote_depth,
            ctx,
            calls,
        );
    }
}
