use crate::application::usecase::rename::reader::{
    explicit_reader_form_kind, explicit_reader_function_lambda_body_children,
};
use crate::application::usecase::rename::selection::list_head;
use crate::domain::common_lisp::common_lisp_operator_head_eq;
use crate::domain::sexpr::{ExpressionKind, ExpressionView, Path, ReaderPrefix};

use super::super::RenameFunctionOccurrence;
use super::super::scope::reader_lambda_body_scope as activate_reader_lambda_body_scope;
use super::core::{
    RenameTraversalMode, TraversalContext, TraversalState, recurse_child,
    recurse_explicit_reader_children,
};

pub(in crate::application::usecase::rename::macrolet) fn collect_explicit_reader_form_renames<
    M: RenameTraversalMode,
>(
    view: &ExpressionView,
    path: &Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    let Some(kind_name) = explicit_reader_form_kind(view) else {
        return false;
    };

    match kind_name.as_str() {
        "quote" => true,
        "function" => {
            M::collect_function_reader_target_renames(view, path, context, state, renames);
            if let Some(children) = explicit_reader_function_lambda_body_children(view) {
                let lambda_scope =
                    activate_reader_lambda_body_scope(state.reader_lambda_body_scope);
                for (child_index, child) in children {
                    let child_path = path.child(1).child(child_index);
                    if M::collect_explicit_function_lambda_atom_renames(
                        child,
                        &child_path,
                        context,
                        lambda_scope,
                        state.quasiquote_depth,
                        renames,
                    ) {
                        continue;
                    }
                    recurse_child::<M>(
                        child,
                        child_path,
                        context,
                        state
                            .with_scopes(lambda_scope, lambda_scope)
                            .with_quasiquote_depth(state.quasiquote_depth),
                        renames,
                    );
                }
            }
            true
        }
        "quasiquote" => {
            recurse_explicit_reader_children::<M>(
                view,
                path,
                context,
                state.with_quasiquote_depth(state.quasiquote_depth + 1),
                renames,
            );
            true
        }
        "unquote" | "unquote-splicing" if state.quasiquote_depth > 0 => {
            recurse_explicit_reader_children::<M>(
                view,
                path,
                context,
                state.with_quasiquote_depth(state.quasiquote_depth - 1),
                renames,
            );
            true
        }
        _ => false,
    }
}

pub(in crate::application::usecase::rename::macrolet) fn collect_reader_lambda_renames<
    M: RenameTraversalMode,
>(
    view: &ExpressionView,
    path: &Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    if view.kind != ExpressionKind::List || !view.reader_prefixes.contains(&ReaderPrefix::Function)
    {
        return false;
    }

    let Some(head) = list_head(view) else {
        return false;
    };
    if !common_lisp_operator_head_eq(head, "lambda") {
        return false;
    }

    let lambda_scope = activate_reader_lambda_body_scope(state.reader_lambda_body_scope);
    for (child_index, child) in view.children.iter().enumerate().skip(2) {
        let child_path = path.child(child_index);
        if M::collect_reader_quoted_lambda_atom_renames(
            child,
            &child_path,
            context,
            lambda_scope,
            state.quasiquote_depth,
            renames,
        ) {
            continue;
        }
        recurse_child::<M>(
            child,
            child_path,
            context,
            state
                .with_scopes(lambda_scope, lambda_scope)
                .with_quasiquote_depth(state.quasiquote_depth),
            renames,
        );
    }
    true
}
