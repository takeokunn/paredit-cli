use anyhow::Result;

use crate::domain::callable_scope::{
    common_lisp_local_callable_form, is_local_callable_bound, local_callable_binding_body_scope,
    local_callable_body_scope, local_callable_names,
};
use crate::domain::common_lisp::{CommonLispLocalCallableForm, common_lisp_symbol_reference_eq};
use crate::domain::dialect::Dialect;
use crate::domain::function_parameter::calls::matches_function_call_view;
use crate::domain::function_parameter::list_edit::{list_head, spans_overlap};
use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

use super::SelectedLocalCallableTraversal;
use super::shared::matched_setf_place_call;

pub(super) fn discover_local_callable_binding_call_paths(
    tree: &SyntaxTree,
    dialect: Dialect,
    definition_span: ByteSpan,
    enclosing_form_span: ByteSpan,
    function_name: &SymbolName,
    form: CommonLispLocalCallableForm,
) -> Result<Vec<Path>> {
    let mut call_paths = Vec::new();
    let context = SelectedLocalCallableTraversal {
        dialect,
        definition_span,
        enclosing_form_span,
        function_name,
        form,
    };

    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let selection = tree.select_path(&path)?;
        let view = selection.view();
        collect_selected_local_callable_binding_call_paths(
            &view,
            path,
            &context,
            false,
            &[],
            &mut call_paths,
        );
    }

    call_paths.sort_by_key(|path| {
        tree.select_path(path)
            .map(|selection| selection.span().start().get())
            .unwrap_or(usize::MAX)
    });
    Ok(call_paths)
}

fn collect_selected_local_callable_binding_call_paths(
    view: &ExpressionView,
    path: Path,
    context: &SelectedLocalCallableTraversal<'_>,
    selected_binding_visible: bool,
    local_callables: &[String],
    output: &mut Vec<Path>,
) {
    if view.span == context.enclosing_form_span
        && list_head(view).is_some_and(|head| {
            common_lisp_local_callable_form(context.dialect, head) == Some(context.form)
        })
    {
        collect_selected_binding_enclosing_form(view, path, context, local_callables, output);
        return;
    }

    if let Some(head) = list_head(view) {
        if let Some(form) = common_lisp_local_callable_form(context.dialect, head) {
            collect_nested_local_callable_paths(
                view,
                path,
                context,
                selected_binding_visible,
                local_callables,
                output,
                form,
            );
            return;
        }
    }

    if selected_binding_visible
        && view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && !spans_overlap(context.definition_span, view.span)
        && matches_function_call_view(view, context.function_name)
        && !is_local_callable_bound(local_callables, context.function_name.as_str())
    {
        output.push(path.clone());
        collect_matched_function_call_descendants(
            view,
            path,
            context,
            selected_binding_visible,
            local_callables,
            output,
        );
        return;
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_selected_local_callable_binding_call_paths(
            child,
            path.child(index),
            context,
            selected_binding_visible,
            local_callables,
            output,
        );
    }
}

fn collect_selected_binding_enclosing_form(
    view: &ExpressionView,
    path: Path,
    context: &SelectedLocalCallableTraversal<'_>,
    local_callables: &[String],
    output: &mut Vec<Path>,
) {
    let local_names = local_callable_names(view)
        .into_iter()
        .filter(|name| !common_lisp_symbol_reference_eq(name, context.function_name.as_str()))
        .collect::<Vec<_>>();
    let mut body_scope = local_callables.to_vec();
    body_scope.extend(local_names);
    let binding_body_visible = matches!(context.form, CommonLispLocalCallableForm::Labels);
    let binding_body_scope = match context.form {
        CommonLispLocalCallableForm::Labels => body_scope.as_slice(),
        CommonLispLocalCallableForm::Flet
        | CommonLispLocalCallableForm::Macrolet
        | CommonLispLocalCallableForm::CompilerMacrolet => local_callables,
    };

    if let Some(bindings) = view.children.get(1) {
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            for (binding_child_index, binding_child) in binding.children.iter().enumerate().skip(2)
            {
                collect_selected_local_callable_binding_call_paths(
                    binding_child,
                    path.descendant([1, binding_index, binding_child_index]),
                    context,
                    binding_body_visible,
                    binding_body_scope,
                    output,
                );
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        collect_selected_local_callable_binding_call_paths(
            child,
            path.child(index),
            context,
            true,
            &body_scope,
            output,
        );
    }
}

fn collect_nested_local_callable_paths(
    view: &ExpressionView,
    path: Path,
    context: &SelectedLocalCallableTraversal<'_>,
    selected_binding_visible: bool,
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
                collect_selected_local_callable_binding_call_paths(
                    binding_child,
                    path.descendant([1, binding_index, binding_child_index]),
                    context,
                    selected_binding_visible,
                    binding_body_scope,
                    output,
                );
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        collect_selected_local_callable_binding_call_paths(
            child,
            path.child(index),
            context,
            selected_binding_visible,
            &body_scope,
            output,
        );
    }
}

fn collect_matched_function_call_descendants(
    view: &ExpressionView,
    path: Path,
    context: &SelectedLocalCallableTraversal<'_>,
    selected_binding_visible: bool,
    local_callables: &[String],
    output: &mut Vec<Path>,
) {
    if let Some(place) = matched_setf_place_call(view, context.function_name) {
        for (index, child) in view.children.iter().enumerate() {
            if index != 1 {
                collect_selected_local_callable_binding_call_paths(
                    child,
                    path.child(index),
                    context,
                    selected_binding_visible,
                    local_callables,
                    output,
                );
            }
        }
        for (index, child) in place.children.iter().enumerate().skip(1) {
            collect_selected_local_callable_binding_call_paths(
                child,
                path.descendant([1, index]),
                context,
                selected_binding_visible,
                local_callables,
                output,
            );
        }
        return;
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_selected_local_callable_binding_call_paths(
            child,
            path.child(index),
            context,
            selected_binding_visible,
            local_callables,
            output,
        );
    }
}
