use crate::application::usecase::callable_scope::is_local_callable_bound;
use crate::application::usecase::rename::reader::{
    explicit_reader_form_kind, explicit_reader_function_lambda_body_children,
};
use crate::domain::common_lisp::common_lisp_symbol_name_eq;
use crate::domain::sexpr::{ExpressionKind, ExpressionView, ReaderPrefix};

use super::super::RenameFunctionOccurrence;
use super::super::target::callable_name_target;
use super::core::{TraversalContext, TraversalState, collect_function_call_head_renames_from_view};

pub(in crate::application::usecase::rename::function) fn collect_function_designator_renames(
    view: &ExpressionView,
    state: &TraversalState<'_>,
    context: &TraversalContext<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    if state.quasiquote_depth == 0 && view.reader_prefixes.contains(&ReaderPrefix::Function) {
        if let Some(target) = callable_name_target(view, &state.path) {
            if common_lisp_symbol_name_eq(target.text, context.from.as_str())
                && !is_local_callable_bound(state.local_callables, target.text)
                && state.shadowed_depth == 0
            {
                renames.push(RenameFunctionOccurrence {
                    path: target.path.to_string(),
                    span: target.span,
                    text: target.text.to_owned(),
                    replacement: context.to.as_str().to_owned(),
                });
                return true;
            }
        }
    }

    false
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
                if let Some(target) = callable_name_target(target, &state.path.child(1)) {
                    if common_lisp_symbol_name_eq(target.text, context.from.as_str())
                        && !is_local_callable_bound(state.local_callables, target.text)
                        && state.shadowed_depth == 0
                    {
                        renames.push(RenameFunctionOccurrence {
                            path: target.path.to_string(),
                            span: target.span,
                            text: target.text.to_owned(),
                            replacement: context.to.as_str().to_owned(),
                        });
                    }
                }
            }
            true
        }
        "function" if state.quasiquote_depth == 0 => {
            if let Some(target) = view.children.get(1) {
                if let Some(target) = callable_name_target(target, &state.path.child(1)) {
                    if common_lisp_symbol_name_eq(target.text, context.from.as_str())
                        && !is_local_callable_bound(state.local_callables, target.text)
                        && state.shadowed_depth == 0
                    {
                        renames.push(RenameFunctionOccurrence {
                            path: target.path.to_string(),
                            span: target.span,
                            text: target.text.to_owned(),
                            replacement: context.to.as_str().to_owned(),
                        });
                    }
                }
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
