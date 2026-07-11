use crate::application::usecase::rename::reader::{
    bare_lambda_body_children, explicit_reader_form_kind,
    explicit_reader_function_lambda_body_children,
};
use crate::domain::sexpr::{ExpressionView, Path};

use super::super::RenameFunctionOccurrence;
use super::super::scope::reader_lambda_body_scope as activate_reader_lambda_body_scope;
use super::core::{RenameTraversalMode, recurse_child, recurse_explicit_reader_children};
use super::state::{TraversalContext, TraversalState};

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
                    let lambda_state = state
                        .with_scopes(lambda_scope, lambda_scope)
                        .with_quasiquote_depth(state.quasiquote_depth);
                    if M::collect_explicit_function_lambda_atom_renames(
                        child,
                        &child_path,
                        context,
                        lambda_state,
                        renames,
                    ) {
                        continue;
                    }
                    recurse_child::<M>(child, child_path, context, lambda_state, renames);
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

/// Handles a bare `(lambda ...)` form directly, not just the `#'(lambda ...)`
/// spelling handled by the "function" arm above; see `bare_lambda_body_children`.
pub(in crate::application::usecase::rename::macrolet) fn collect_reader_lambda_renames<
    M: RenameTraversalMode,
>(
    view: &ExpressionView,
    path: &Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    let Some(children) = bare_lambda_body_children(view) else {
        return false;
    };

    let lambda_scope = activate_reader_lambda_body_scope(state.reader_lambda_body_scope);
    for (child_index, child) in children {
        let child_path = path.child(child_index);
        let lambda_state = state
            .with_scopes(lambda_scope, lambda_scope)
            .with_quasiquote_depth(state.quasiquote_depth);
        if M::collect_reader_quoted_lambda_atom_renames(
            child,
            &child_path,
            context,
            lambda_state,
            renames,
        ) {
            continue;
        }
        recurse_child::<M>(child, child_path, context, lambda_state, renames);
    }
    true
}
