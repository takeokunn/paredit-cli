use crate::domain::common_lisp::CommonLispLocalCallableForm;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path, SymbolName};

use crate::domain::rename::reader::apply_reader_prefix_context;

use super::super::RenameFunctionOccurrence;
use super::super::scope::{
    LocalCallableRenameKind, MacroletRenameScope, symbol_macrolet_shadowing_scope,
};
use super::local_callable::{LocalCallableTraversal, collect_local_callable_or_definition};
use super::reader::{collect_explicit_reader_form_renames, collect_reader_lambda_renames};
use super::state::{TraversalContext, TraversalState};

pub(in crate::domain::rename::macrolet) trait RenameTraversalMode {
    fn collect_pre_reader_renames(
        _view: &ExpressionView,
        _path: &Path,
        _context: TraversalContext<'_>,
        _state: TraversalState,
        _renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        false
    }

    fn collect_function_reader_target_renames(
        _view: &ExpressionView,
        _path: &Path,
        _context: TraversalContext<'_>,
        _state: TraversalState,
        _renames: &mut Vec<RenameFunctionOccurrence>,
    ) {
    }

    fn collect_list_head_renames(
        _view: &ExpressionView,
        _path: &Path,
        _context: TraversalContext<'_>,
        _state: TraversalState,
        _renames: &mut Vec<RenameFunctionOccurrence>,
    ) {
    }

    fn collect_binding_name_renames(
        _binding: &ExpressionView,
        _binding_index: usize,
        _path: &Path,
        _form: CommonLispLocalCallableForm,
        _context: TraversalContext<'_>,
        _state: TraversalState,
        _renames: &mut Vec<RenameFunctionOccurrence>,
    ) {
    }

    fn collect_explicit_function_lambda_atom_renames(
        _child: &ExpressionView,
        _child_path: &Path,
        _context: TraversalContext<'_>,
        _state: TraversalState,
        _renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        false
    }

    fn collect_reader_quoted_lambda_atom_renames(
        _child: &ExpressionView,
        _child_path: &Path,
        _context: TraversalContext<'_>,
        _state: TraversalState,
        _renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        false
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "recursive traversal threads scope, quasiquote state, and accumulator"
)]
pub(in crate::domain::rename::macrolet) fn collect_renames_from_view<
    M: RenameTraversalMode,
>(
    view: &ExpressionView,
    path: Path,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    kind: LocalCallableRenameKind,
    scope: MacroletRenameScope,
    reader_lambda_body_scope: MacroletRenameScope,
    quasiquote_depth: usize,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let context = TraversalContext {
        dialect,
        from,
        to,
        kind,
    };
    let state = TraversalState {
        scope,
        reader_lambda_body_scope,
        quasiquote_depth,
    };
    collect_renames_from_view_with_mode::<M>(view, path, context, state, renames);
}

pub(in crate::domain::rename::macrolet) fn collect_renames_from_view_with_mode<
    M: RenameTraversalMode,
>(
    view: &ExpressionView,
    path: Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let scope = symbol_macrolet_shadowing_scope(state.scope, view, context.from);
    let state = state.with_scope(scope);

    if M::collect_pre_reader_renames(view, &path, context, state, renames) {
        return;
    }

    let Some(quasiquote_depth) = apply_reader_prefix_context(view, state.quasiquote_depth) else {
        return;
    };
    let state = state.with_quasiquote_depth(quasiquote_depth);

    if collect_explicit_reader_form_renames::<M>(view, &path, context, state, renames) {
        return;
    }

    if collect_reader_lambda_renames::<M>(view, &path, context, state, renames) {
        return;
    }

    if state.quasiquote_depth > 0 {
        recurse_children::<M>(view, &path, context, state, renames);
        return;
    }

    let definition_body_range =
        match collect_local_callable_or_definition::<M>(view, &path, context, state, renames) {
            LocalCallableTraversal::Handled => return,
            LocalCallableTraversal::DefinitionBody(range) => range,
        };

    for (index, child) in view.children.iter().enumerate() {
        if let Some(range) = definition_body_range {
            if !range.contains_child(index) {
                continue;
            }
        }
        recurse_child::<M>(
            child,
            path.child(index),
            context,
            state.with_quasiquote_depth(0),
            renames,
        );
    }
}

fn recurse_children<M: RenameTraversalMode>(
    view: &ExpressionView,
    path: &Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    for (index, child) in view.children.iter().enumerate() {
        recurse_child::<M>(child, path.child(index), context, state, renames);
    }
}

pub(in crate::domain::rename::macrolet) fn recurse_explicit_reader_children<
    M: RenameTraversalMode,
>(
    view: &ExpressionView,
    path: &Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    for (index, child) in view.children.iter().enumerate().skip(1) {
        recurse_child::<M>(child, path.child(index), context, state, renames);
    }
}

pub(in crate::domain::rename::macrolet) fn recurse_child<M: RenameTraversalMode>(
    child: &ExpressionView,
    child_path: Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    collect_renames_from_view_with_mode::<M>(child, child_path, context, state, renames);
}
