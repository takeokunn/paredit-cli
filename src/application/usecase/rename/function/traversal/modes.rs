use crate::application::usecase::callable_scope::{
    common_lisp_local_callable_form, local_callable_binding_body_scope, local_callable_body_scope,
};
use crate::domain::common_lisp::CommonLispOperator;
use crate::domain::sexpr::ExpressionView;

use super::super::RenameFunctionOccurrence;
use super::core::{TraversalContext, TraversalState, collect_function_call_head_renames_from_view};

pub(in crate::application::usecase::rename::function) fn collect_local_callable_function_call_renames(
    view: &ExpressionView,
    head: &str,
    context: &TraversalContext<'_>,
    state: TraversalState<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    let Some(form) = common_lisp_local_callable_form(context.dialect, head) else {
        return false;
    };

    let body_scope = local_callable_body_scope(state.local_callables, view);

    if let Some(bindings) = view.children.get(1) {
        let binding_body_scope =
            local_callable_binding_body_scope(form, state.local_callables, &body_scope);
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                collect_function_call_head_renames_from_view(
                    child,
                    context,
                    TraversalState {
                        path: state.path.descendant([1, binding_index, child_index]),
                        local_callables: binding_body_scope,
                        quasiquote_depth: 0,
                        in_macro_expander: state.in_macro_expander || form.is_macro(),
                        shadowed_depth: state.shadowed_depth,
                    },
                    renames,
                );
            }
        }
    }

    collect_body_forms(
        view,
        context,
        state.with_local_callables(&body_scope),
        renames,
    );
    true
}

pub(in crate::application::usecase::rename::function) fn collect_symbol_macrolet_function_call_renames(
    view: &ExpressionView,
    head: &str,
    context: &TraversalContext<'_>,
    state: TraversalState<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    if CommonLispOperator::from_head(head) != Some(CommonLispOperator::SymbolMacrolet) {
        return false;
    }

    if let Some(bindings) = view.children.get(1) {
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            for (child_index, child) in binding.children.iter().enumerate().skip(1) {
                collect_function_call_head_renames_from_view(
                    child,
                    context,
                    state
                        .with_quasiquote_depth(0)
                        .with_path(state.path.descendant([1, binding_index, child_index])),
                    renames,
                );
            }
        }
    }

    collect_body_forms(view, context, state, renames);
    true
}

fn collect_body_forms(
    view: &ExpressionView,
    context: &TraversalContext<'_>,
    state: TraversalState<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    for (index, child) in view.children.iter().enumerate().skip(2) {
        collect_function_call_head_renames_from_view(
            child,
            context,
            state.with_path(state.path.child(index)),
            renames,
        );
    }
}
