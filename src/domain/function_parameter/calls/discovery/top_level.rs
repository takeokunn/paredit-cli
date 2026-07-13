use anyhow::Result;

use crate::domain::callable_scope::{
    common_lisp_local_callable_form, is_local_callable_bound, local_callable_binding_body_scope,
    local_callable_body_scope,
};
use crate::domain::common_lisp::CommonLispLocalCallableForm;
use crate::domain::dialect::Dialect;
use crate::domain::function_parameter::calls::matches_function_call_view;
use crate::domain::function_parameter::list_edit::{list_head, spans_overlap};
use crate::domain::sexpr::{
    Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

use super::FunctionCallTraversal;
use super::shared::matched_setf_place_call;

pub(super) fn discover_function_call_paths(
    tree: &SyntaxTree,
    dialect: Dialect,
    definition_span: crate::domain::sexpr::ByteSpan,
    function_name: &SymbolName,
) -> Result<Vec<Path>> {
    let mut call_paths = Vec::new();
    let context = FunctionCallTraversal {
        dialect,
        definition_span,
        function_name,
    };

    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let selection = tree.select_path(&path)?;
        let view = selection.view();
        collect_function_call_paths(&view, path, &context, &[], &mut call_paths);
    }

    call_paths.sort_by_key(|path| {
        tree.select_path(path)
            .map(|selection| selection.span().start().get())
            .unwrap_or(usize::MAX)
    });
    Ok(call_paths)
}

fn collect_function_call_paths(
    view: &ExpressionView,
    path: Path,
    context: &FunctionCallTraversal<'_>,
    local_callables: &[String],
    output: &mut Vec<Path>,
) {
    if let Some(head) = list_head(view) {
        if let Some(form) = common_lisp_local_callable_form(context.dialect, head) {
            collect_local_callable_function_call_paths(
                view,
                path,
                context,
                local_callables,
                output,
                form,
            );
            return;
        }
    }

    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && !spans_overlap(context.definition_span, view.span)
        && matches_function_call_view(view, context.function_name)
        && !is_local_callable_bound(local_callables, context.function_name.as_str())
    {
        output.push(path.clone());
        collect_matched_top_level_function_call_descendants(
            view,
            path,
            context,
            local_callables,
            output,
        );
        return;
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_function_call_paths(child, path.child(index), context, local_callables, output);
    }
}

fn collect_local_callable_function_call_paths(
    view: &ExpressionView,
    path: Path,
    context: &FunctionCallTraversal<'_>,
    local_callables: &[String],
    output: &mut Vec<Path>,
    form: CommonLispLocalCallableForm,
) {
    let body_scope = local_callable_body_scope(local_callables, view);
    let binding_body_scope = local_callable_binding_body_scope(form, local_callables, &body_scope);

    if let Some(bindings) = view.children.get(1) {
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            for (binding_child_index, binding_child) in binding.children.iter().enumerate().skip(2)
            {
                collect_function_call_paths(
                    binding_child,
                    path.descendant([1, binding_index, binding_child_index]),
                    context,
                    binding_body_scope,
                    output,
                );
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        collect_function_call_paths(child, path.child(index), context, &body_scope, output);
    }
}

fn collect_matched_top_level_function_call_descendants(
    view: &ExpressionView,
    path: Path,
    context: &FunctionCallTraversal<'_>,
    local_callables: &[String],
    output: &mut Vec<Path>,
) {
    if let Some(place) = matched_setf_place_call(view, context.function_name) {
        for (index, child) in view.children.iter().enumerate() {
            if index != 1 {
                collect_function_call_paths(
                    child,
                    path.child(index),
                    context,
                    local_callables,
                    output,
                );
            }
        }
        for (index, child) in place.children.iter().enumerate().skip(1) {
            collect_function_call_paths(
                child,
                path.descendant([1, index]),
                context,
                local_callables,
                output,
            );
        }
        return;
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_function_call_paths(child, path.child(index), context, local_callables, output);
    }
}
