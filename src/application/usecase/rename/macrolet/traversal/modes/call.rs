use crate::application::usecase::rename::function::target::callable_name_target;
use crate::application::usecase::rename::macrolet::RenameFunctionOccurrence;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::sexpr::{ExpressionView, Path};

use super::super::super::reader::{
    atom_symbol_span, atom_symbol_text, collect_local_function_designator_renames,
    push_callable_target_rename_if_match,
};
use super::super::super::scope::LocalCallableRenameKind;
use super::super::core::RenameTraversalMode;
use super::super::state::{TraversalContext, TraversalState};
use super::common::{callable_list_head_target, collect_active_atom_rename};

pub(in crate::application::usecase::rename::macrolet) struct CallTraversal;

impl RenameTraversalMode for CallTraversal {
    fn collect_pre_reader_renames(
        view: &ExpressionView,
        path: &Path,
        context: TraversalContext<'_>,
        state: TraversalState,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        collect_local_function_designator_renames(
            view,
            path,
            context.from,
            context.to,
            context.kind,
            state.scope,
            state.quasiquote_depth,
            renames,
        )
    }

    fn collect_function_reader_target_renames(
        view: &ExpressionView,
        path: &Path,
        context: TraversalContext<'_>,
        state: TraversalState,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) {
        if context.kind != LocalCallableRenameKind::Function || !state.allows_current_scope_rename()
        {
            return;
        }

        if let Some(target) = view.children.get(1) {
            if let Some(target) = callable_name_target(target, &path.child(1)) {
                push_callable_target_rename_if_match(target, context.from, context.to, renames);
            }
        }
    }

    fn collect_list_head_renames(
        view: &ExpressionView,
        path: &Path,
        context: TraversalContext<'_>,
        state: TraversalState,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) {
        if context.kind == LocalCallableRenameKind::Function
            && state.allows_current_scope_rename()
            && let Some(target) = callable_list_head_target(view, path)
        {
            push_callable_target_rename_if_match(target, context.from, context.to, renames);
        }

        let Some(head) = crate::application::usecase::rename::selection::list_head(view) else {
            return;
        };
        if !common_lisp_symbol_reference_eq(head, context.from.as_str())
            || !state.allows_current_scope_rename()
        {
            return;
        }

        if let Some(head_view) = view.children.first() {
            renames.push(RenameFunctionOccurrence {
                path: path.child(0).to_string(),
                span: atom_symbol_span(head_view).unwrap_or(head_view.span),
                text: atom_symbol_text(head_view).unwrap_or(head).to_owned(),
                replacement: context.to.as_str().to_owned(),
            });
        }
    }

    fn collect_explicit_function_lambda_atom_renames(
        child: &ExpressionView,
        child_path: &Path,
        context: TraversalContext<'_>,
        state: TraversalState,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        collect_active_atom_rename(child, child_path, context, state, state.scope, renames)
    }

    fn collect_reader_quoted_lambda_atom_renames(
        child: &ExpressionView,
        child_path: &Path,
        context: TraversalContext<'_>,
        state: TraversalState,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        collect_active_atom_rename(child, child_path, context, state, state.scope, renames)
    }
}
