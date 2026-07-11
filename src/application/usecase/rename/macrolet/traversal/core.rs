use crate::application::usecase::callable_scope::{
    common_lisp_local_callable_form, local_callable_names,
};
use crate::application::usecase::rename::selection::list_head;
use crate::domain::common_lisp::CommonLispLocalCallableForm;
use crate::domain::definition::definition_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path, SymbolName};

use crate::application::usecase::rename::reader::apply_reader_prefix_context;

use super::super::RenameFunctionOccurrence;
use super::super::scope::{
    LocalCallableRenameKind, MacroletRenameScope, local_callable_scopes,
    symbol_macrolet_shadowing_scope, target_binding_presence,
};
use super::reader::{collect_explicit_reader_form_renames, collect_reader_lambda_renames};

#[derive(Clone, Copy)]
pub(in crate::application::usecase::rename::macrolet) struct TraversalContext<'a> {
    pub(super) dialect: Dialect,
    pub(super) from: &'a SymbolName,
    pub(super) to: &'a SymbolName,
    pub(super) kind: LocalCallableRenameKind,
}

#[derive(Clone, Copy)]
pub(in crate::application::usecase::rename::macrolet) struct TraversalState {
    pub(super) scope: MacroletRenameScope,
    pub(super) reader_lambda_body_scope: MacroletRenameScope,
    pub(super) quasiquote_depth: usize,
}

impl TraversalState {
    pub(super) fn with_scope(&self, scope: MacroletRenameScope) -> Self {
        Self {
            scope,
            reader_lambda_body_scope: self.reader_lambda_body_scope,
            quasiquote_depth: self.quasiquote_depth,
        }
    }

    pub(super) fn with_scopes(
        &self,
        scope: MacroletRenameScope,
        reader_lambda_body_scope: MacroletRenameScope,
    ) -> Self {
        Self {
            scope,
            reader_lambda_body_scope,
            quasiquote_depth: self.quasiquote_depth,
        }
    }

    pub(super) fn with_quasiquote_depth(&self, quasiquote_depth: usize) -> Self {
        Self {
            scope: self.scope,
            reader_lambda_body_scope: self.reader_lambda_body_scope,
            quasiquote_depth,
        }
    }
}

pub(in crate::application::usecase::rename::macrolet) trait RenameTraversalMode {
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
        _scope: MacroletRenameScope,
        _quasiquote_depth: usize,
        _renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        false
    }

    fn collect_reader_quoted_lambda_atom_renames(
        _child: &ExpressionView,
        _child_path: &Path,
        _context: TraversalContext<'_>,
        _scope: MacroletRenameScope,
        _quasiquote_depth: usize,
        _renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        false
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "recursive traversal threads scope, quasiquote state, and accumulator"
)]
pub(in crate::application::usecase::rename::macrolet) fn collect_renames_from_view<
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

pub(in crate::application::usecase::rename::macrolet) fn collect_renames_from_view_with_mode<
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

    let mut definition_body_range = None;

    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        if let Some(head) = list_head(view) {
            if let Some(form) = common_lisp_local_callable_form(context.dialect, head) {
                collect_local_callable_form_renames::<M>(
                    view, &path, context, state, form, renames,
                );
                return;
            }

            let shape = definition_shape(context.dialect, view, head);
            if shape.is_none() {
                M::collect_list_head_renames(view, &path, context, state, renames);
            }
            if let Some(shape) = shape {
                definition_body_range = Some(shape.body_range());
            }
        } else {
            M::collect_list_head_renames(view, &path, context, state, renames);
        }
    }

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

fn collect_local_callable_form_renames<M: RenameTraversalMode>(
    view: &ExpressionView,
    path: &Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    form: CommonLispLocalCallableForm,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let local_names = local_callable_names(view);
    let target_binding = target_binding_presence(&local_names, context.from);
    let scopes = local_callable_scopes(state.scope, context.kind, form, target_binding);

    if let Some(bindings) = view.children.get(1) {
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            M::collect_binding_name_renames(
                binding,
                binding_index,
                path,
                form,
                context,
                state,
                renames,
            );

            for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                recurse_child::<M>(
                    child,
                    path.descendant([1, binding_index, child_index]),
                    context,
                    state
                        .with_scopes(scopes.binding_body, scopes.body)
                        .with_quasiquote_depth(0),
                    renames,
                );
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        recurse_child::<M>(
            child,
            path.child(index),
            context,
            state
                .with_scopes(scopes.body, scopes.body)
                .with_quasiquote_depth(0),
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

pub(in crate::application::usecase::rename::macrolet) fn recurse_explicit_reader_children<
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

pub(in crate::application::usecase::rename::macrolet) fn recurse_child<M: RenameTraversalMode>(
    child: &ExpressionView,
    child_path: Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    collect_renames_from_view_with_mode::<M>(child, child_path, context, state, renames);
}
