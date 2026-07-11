use crate::application::usecase::rename::reader::{
    bare_lambda_body_children, explicit_reader_form_kind,
    explicit_reader_function_lambda_body_children,
};
use crate::domain::common_lisp::common_lisp_symbol_name_eq;
use crate::domain::sexpr::{ExpressionKind, ExpressionView, ReaderPrefix};

use super::super::RenameFunctionOccurrence;
use super::super::target::callable_name_target;
use super::core::{
    TraversalContext, TraversalState, allows_function_reference_rename,
    collect_function_call_head_renames_from_view,
};

fn collect_callable_target_rename(
    target_view: &ExpressionView,
    target_path: crate::domain::sexpr::Path,
    state: &TraversalState<'_>,
    context: &TraversalContext<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    let Some(target) = callable_name_target(target_view, &target_path) else {
        return false;
    };

    if !common_lisp_symbol_name_eq(target.text, context.from.as_str())
        || !allows_function_reference_rename(state, target.text)
    {
        return false;
    }

    renames.push(RenameFunctionOccurrence {
        path: target.path.to_string(),
        span: target.span,
        text: target.text.to_owned(),
        replacement: context.to.as_str().to_owned(),
    });
    true
}

pub(in crate::application::usecase::rename::function) fn collect_function_designator_renames(
    view: &ExpressionView,
    state: &TraversalState<'_>,
    context: &TraversalContext<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    if state.quasiquote_depth == 0 && view.reader_prefixes.contains(&ReaderPrefix::Function) {
        return collect_callable_target_rename(view, state.path.clone(), state, context, renames);
    }

    false
}

/// Handles a bare `(lambda ...)` form directly, skipping its parameter list
/// the same way the `#'(lambda ...)` case below skips it via
/// `explicit_reader_function_lambda_body_children`; see `bare_lambda_body_children`.
pub(in crate::application::usecase::rename::function) fn collect_bare_lambda_call_renames(
    view: &ExpressionView,
    context: &TraversalContext<'_>,
    state: &TraversalState<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    let Some(children) = bare_lambda_body_children(view) else {
        return false;
    };

    for (child_index, child) in children {
        collect_function_call_head_renames_from_view(
            child,
            context,
            state.with_path(state.path.child(child_index)),
            renames,
        );
    }
    true
}

pub(in crate::application::usecase::rename::function) fn collect_explicit_reader_form_call_renames(
    view: &ExpressionView,
    context: &TraversalContext<'_>,
    state: TraversalState<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    if view.kind != ExpressionKind::List || view.children.len() < 2 {
        return false;
    }

    let Some(head) = explicit_reader_form_kind(view) else {
        return false;
    };

    match head.as_str() {
        "quote" => true,
        "macro-function" | "compiler-macro-function" | "symbol-function" | "fdefinition"
            if state.quasiquote_depth == 0 =>
        {
            if let Some(target) = view.children.get(1) {
                collect_callable_target_rename(
                    target,
                    state.path.child(1),
                    &state,
                    context,
                    renames,
                );
            }
            true
        }
        "function" if state.quasiquote_depth == 0 => {
            if let Some(target) = view.children.get(1) {
                collect_callable_target_rename(
                    target,
                    state.path.child(1),
                    &state,
                    context,
                    renames,
                );
            }
            if let Some(children) = explicit_reader_function_lambda_body_children(view) {
                for (child_index, child) in children {
                    collect_function_call_head_renames_from_view(
                        child,
                        context,
                        state.with_path(state.path.child(1).child(child_index)),
                        renames,
                    );
                }
            }
            true
        }
        "function" => true,
        "quasiquote" => {
            for (index, child) in view.children.iter().enumerate().skip(1) {
                collect_function_call_head_renames_from_view(
                    child,
                    context,
                    state
                        .with_quasiquote_depth(state.quasiquote_depth + 1)
                        .with_path(state.path.child(index)),
                    renames,
                );
            }
            true
        }
        "unquote" | "unquote-splicing" if state.quasiquote_depth > 0 => {
            for (index, child) in view.children.iter().enumerate().skip(1) {
                collect_function_call_head_renames_from_view(
                    child,
                    context,
                    state
                        .with_quasiquote_depth(state.quasiquote_depth - 1)
                        .with_path(state.path.child(index)),
                    renames,
                );
            }
            true
        }
        _ => false,
    }
}
