use anyhow::Result;

use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::definition::{definition_shape, is_macro_expander_definition};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

use crate::domain::callable_scope::is_local_callable_bound;
use crate::domain::rename::reader::{
    apply_reader_prefix_context, atom_symbol_span, atom_symbol_text,
};

use super::super::super::selection::list_head;
use super::super::RenameFunctionOccurrence;
use super::modes::{
    collect_local_callable_function_call_renames, collect_symbol_macrolet_function_call_renames,
};
use super::reader::{
    collect_bare_lambda_call_renames, collect_explicit_reader_form_call_renames,
    collect_function_designator_renames, collect_quoted_symbol_designator_rename,
};

pub(in crate::domain::rename::function) struct TraversalContext<'a> {
    pub(super) dialect: Dialect,
    pub(super) from: &'a SymbolName,
    pub(super) to: &'a SymbolName,
}

pub(in crate::domain::rename::function) struct TraversalState<'a> {
    pub(super) path: Path,
    pub(super) local_callables: &'a [String],
    pub(super) quasiquote_depth: usize,
    pub(super) in_macro_expander: bool,
    pub(super) shadowed_depth: usize,
}

impl<'a> TraversalState<'a> {
    pub(super) fn with_path(&self, path: Path) -> Self {
        Self {
            path,
            local_callables: self.local_callables,
            quasiquote_depth: self.quasiquote_depth,
            in_macro_expander: self.in_macro_expander,
            shadowed_depth: self.shadowed_depth,
        }
    }

    pub(super) fn with_quasiquote_depth(&self, quasiquote_depth: usize) -> Self {
        Self {
            path: self.path.clone(),
            local_callables: self.local_callables,
            quasiquote_depth,
            in_macro_expander: self.in_macro_expander,
            shadowed_depth: self.shadowed_depth,
        }
    }

    pub(super) fn with_local_callables(&self, local_callables: &'a [String]) -> Self {
        Self {
            path: self.path.clone(),
            local_callables,
            quasiquote_depth: self.quasiquote_depth,
            in_macro_expander: self.in_macro_expander,
            shadowed_depth: self.shadowed_depth,
        }
    }

    pub(super) fn in_macro_expander(&self) -> Self {
        Self {
            path: self.path.clone(),
            local_callables: self.local_callables,
            quasiquote_depth: self.quasiquote_depth,
            in_macro_expander: true,
            shadowed_depth: self.shadowed_depth,
        }
    }
}

pub(in crate::domain::rename::function) fn allows_function_reference_rename(
    state: &TraversalState<'_>,
    target_text: &str,
) -> bool {
    (!is_local_callable_bound(state.local_callables, target_text) && state.shadowed_depth == 0)
        || is_package_qualified_callable(target_text)
}

fn is_package_qualified_callable(target_text: &str) -> bool {
    let Some((package_name, symbol_name)) = target_text.split_once(':') else {
        return false;
    };

    !target_text.starts_with(':') && !package_name.is_empty() && !symbol_name.is_empty()
}

pub fn collect_function_call_head_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<RenameFunctionOccurrence>> {
    let context = TraversalContext { dialect, from, to };
    let mut renames = Vec::new();

    for (index, _) in tree.root_children().iter().enumerate() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collect_function_call_head_renames_from_view(
            &view,
            &context,
            TraversalState {
                path,
                local_callables: &[],
                quasiquote_depth: 0,
                in_macro_expander: false,
                shadowed_depth: 0,
            },
            &mut renames,
        );
    }

    Ok(renames)
}

pub(in crate::domain::rename::function) fn collect_function_call_head_renames_from_view(
    view: &ExpressionView,
    context: &TraversalContext<'_>,
    state: TraversalState<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let Some(quasiquote_depth) = apply_reader_prefix_context(view, state.quasiquote_depth) else {
        collect_quoted_symbol_designator_rename(view, &state, context, renames);
        return;
    };

    let reader_state = state.with_quasiquote_depth(quasiquote_depth);
    if collect_function_designator_renames(view, &reader_state, context, renames) {
        return;
    }

    if collect_bare_lambda_call_renames(view, context, &reader_state, renames) {
        return;
    }

    if collect_explicit_reader_form_call_renames(view, context, reader_state, renames) {
        return;
    }

    if quasiquote_depth > 0 && !state.in_macro_expander {
        collect_children(
            view,
            context,
            state.with_quasiquote_depth(quasiquote_depth),
            renames,
        );
        return;
    }

    let mut definition_body_range = None;
    let mut macro_expander_body_range = None;

    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        if let Some(head) = list_head(view) {
            if collect_local_callable_function_call_renames(
                view,
                head,
                context,
                state.with_quasiquote_depth(0),
                renames,
            ) {
                return;
            }

            if collect_symbol_macrolet_function_call_renames(
                view,
                head,
                context,
                state.with_quasiquote_depth(0),
                renames,
            ) {
                return;
            }

            let shape = definition_shape(context.dialect, view, head);
            if common_lisp_symbol_reference_eq(head, context.from.as_str())
                && shape.is_none()
                && allows_function_reference_rename(&state, head)
            {
                if let Some(head_view) = view.children.first() {
                    renames.push(RenameFunctionOccurrence {
                        path: state.path.child(0).to_string(),
                        span: atom_symbol_span(head_view).unwrap_or(head_view.span),
                        text: atom_symbol_text(head_view).unwrap_or(head).to_owned(),
                        replacement: context.to.as_str().to_owned(),
                    });
                }
            }
            if let Some(shape) = shape {
                definition_body_range = Some(shape.body_range());
                if is_macro_expander_definition(context.dialect, head) {
                    macro_expander_body_range = definition_body_range;
                }
            }
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        if let Some(range) = definition_body_range {
            if !range.contains_child(index) {
                continue;
            }
        }
        let child_state = state
            .with_quasiquote_depth(0)
            .with_path(state.path.child(index));
        let child_state =
            if macro_expander_body_range.is_some_and(|range| range.contains_child(index)) {
                child_state.in_macro_expander()
            } else {
                child_state
            };
        collect_function_call_head_renames_from_view(child, context, child_state, renames);
    }
}

pub(in crate::domain::rename::function) fn collect_children(
    view: &ExpressionView,
    context: &TraversalContext<'_>,
    state: TraversalState<'_>,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    for (index, child) in view.children.iter().enumerate() {
        collect_function_call_head_renames_from_view(
            child,
            context,
            state.with_path(state.path.child(index)),
            renames,
        );
    }
}
