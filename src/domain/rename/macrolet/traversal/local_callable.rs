use crate::domain::callable_scope::{common_lisp_local_callable_form, local_callable_names};
use crate::domain::common_lisp::CommonLispLocalCallableForm;
use crate::domain::definition::{DefinitionBodyRange, definition_shape};
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path};

use super::super::RenameFunctionOccurrence;
use super::super::scope::{local_callable_scopes, target_binding_presence};
use super::core::{RenameTraversalMode, recurse_child};
use super::state::{TraversalContext, TraversalState};

pub(in crate::domain::rename::macrolet) enum LocalCallableTraversal {
    Handled,
    DefinitionBody(Option<DefinitionBodyRange>),
}

pub(in crate::domain::rename::macrolet) fn collect_local_callable_or_definition<
    M: RenameTraversalMode,
>(
    view: &ExpressionView,
    path: &Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> LocalCallableTraversal {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return LocalCallableTraversal::DefinitionBody(None);
    }

    let Some(head) = crate::domain::rename::selection::list_head(view) else {
        M::collect_list_head_renames(view, path, context, state, renames);
        return LocalCallableTraversal::DefinitionBody(None);
    };

    if let Some(form) = common_lisp_local_callable_form(context.dialect, head) {
        collect_local_callable_form_renames::<M>(view, path, context, state, form, renames);
        return LocalCallableTraversal::Handled;
    }

    let shape = definition_shape(context.dialect, view, head);
    if shape.is_none() {
        M::collect_list_head_renames(view, path, context, state, renames);
    }
    LocalCallableTraversal::DefinitionBody(shape.map(|shape| shape.body_range()))
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
