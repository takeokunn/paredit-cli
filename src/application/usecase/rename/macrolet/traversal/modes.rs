use crate::domain::common_lisp::{common_lisp_operator_head_eq, common_lisp_symbol_name_eq};
use crate::domain::sexpr::{ExpressionKind, ExpressionView, Path};

use super::super::RenameFunctionOccurrence;
use super::super::reader::{
    atom_symbol_span, atom_symbol_text, collect_local_function_designator_renames,
    push_atom_rename_if_match,
};
use super::super::scope::{LocalCallableRenameKind, MacroletRenameScope};
use super::core::{RenameTraversalMode, TraversalContext, TraversalState};

pub(in crate::application::usecase::rename::macrolet) struct BindingTraversal;

impl RenameTraversalMode for BindingTraversal {
    fn collect_binding_name_renames(
        binding: &ExpressionView,
        binding_index: usize,
        path: &Path,
        form: crate::domain::common_lisp::CommonLispLocalCallableForm,
        context: TraversalContext<'_>,
        state: TraversalState,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) {
        if !context.kind.matches_target_form(form)
            || state.scope.is_target_active()
            || state.scope.is_shadowed()
        {
            return;
        }

        let Some((name_view, name_path)) =
            callable_binding_name_target(binding, path, binding_index)
        else {
            return;
        };

        let Some(name_text) = atom_symbol_text(name_view) else {
            return;
        };
        if !common_lisp_symbol_name_eq(name_text, context.from.as_str()) {
            return;
        }

        renames.push(RenameFunctionOccurrence {
            path: name_path.to_string(),
            span: name_view.span,
            text: name_text.to_owned(),
            replacement: context.to.as_str().to_owned(),
        });
    }

    fn collect_explicit_function_lambda_atom_renames(
        child: &ExpressionView,
        child_path: &Path,
        context: TraversalContext<'_>,
        scope: MacroletRenameScope,
        quasiquote_depth: usize,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        collect_active_atom_rename(child, child_path, context, scope, quasiquote_depth, renames)
    }
}

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
        if context.kind != LocalCallableRenameKind::Function
            || state.quasiquote_depth > 0
            || !state.scope.is_target_active()
            || state.scope.is_shadowed()
        {
            return;
        }

        if let Some(target) = view.children.get(1) {
            if let Some(text) = atom_symbol_text(target) {
                if common_lisp_symbol_name_eq(text, context.from.as_str()) {
                    renames.push(RenameFunctionOccurrence {
                        path: path.child(1).to_string(),
                        span: atom_symbol_span(target).unwrap_or(target.span),
                        text: text.to_owned(),
                        replacement: context.to.as_str().to_owned(),
                    });
                }
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
        if let Some((target, target_path)) = callable_list_head_target(view, path) {
            if common_lisp_symbol_name_eq(
                target.text.as_deref().unwrap_or(""),
                context.from.as_str(),
            ) && state.scope.is_target_active()
                && !state.scope.is_shadowed()
            {
                renames.push(RenameFunctionOccurrence {
                    path: target_path.to_string(),
                    span: target.span,
                    text: target.text.as_deref().unwrap_or_default().to_owned(),
                    replacement: context.to.as_str().to_owned(),
                });
            }
        }

        let Some(head) = crate::application::usecase::rename::selection::list_head(view) else {
            return;
        };
        if !common_lisp_symbol_name_eq(head, context.from.as_str())
            || !state.scope.is_target_active()
            || state.scope.is_shadowed()
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
        scope: MacroletRenameScope,
        quasiquote_depth: usize,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        collect_active_atom_rename(child, child_path, context, scope, quasiquote_depth, renames)
    }

    fn collect_reader_quoted_lambda_atom_renames(
        child: &ExpressionView,
        child_path: &Path,
        context: TraversalContext<'_>,
        scope: MacroletRenameScope,
        quasiquote_depth: usize,
        renames: &mut Vec<RenameFunctionOccurrence>,
    ) -> bool {
        collect_active_atom_rename(child, child_path, context, scope, quasiquote_depth, renames)
    }
}

fn collect_active_atom_rename(
    child: &ExpressionView,
    child_path: &Path,
    context: TraversalContext<'_>,
    scope: MacroletRenameScope,
    quasiquote_depth: usize,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    if child.kind == ExpressionKind::Atom
        && quasiquote_depth == 0
        && scope.is_target_active()
        && !scope.is_shadowed()
    {
        return push_atom_rename_if_match(child, child_path, context.from, context.to, renames);
    }
    false
}

fn callable_binding_name_target<'a>(
    binding: &'a ExpressionView,
    path: &Path,
    binding_index: usize,
) -> Option<(&'a ExpressionView, Path)> {
    let name_view = binding.children.first()?;
    if name_view.kind == ExpressionKind::Atom {
        return Some((name_view, path.descendant([1, binding_index, 0])));
    }

    let head = name_view.children.first()?.text.as_deref()?;
    if !common_lisp_operator_head_eq(head, "setf") {
        return None;
    }

    let target = name_view.children.get(1)?;
    if target.kind != ExpressionKind::Atom {
        return None;
    }

    Some((target, path.descendant([1, binding_index, 0, 1])))
}

fn callable_list_head_target<'a>(
    view: &'a ExpressionView,
    path: &Path,
) -> Option<(&'a ExpressionView, Path)> {
    let head = view.children.first()?;
    if head.kind != ExpressionKind::List {
        return None;
    }

    let operator = head.children.first()?.text.as_deref()?;
    if !common_lisp_operator_head_eq(operator, "setf") {
        return None;
    }

    let target = head.children.get(1)?;
    if target.kind != ExpressionKind::Atom {
        return None;
    }

    Some((target, path.child(0).child(1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::usecase::rename::macrolet::scope::{
        TargetBindingPresence, local_callable_scopes,
    };
    use crate::domain::common_lisp::CommonLispLocalCallableForm;
    use crate::domain::sexpr::{Path, SyntaxTree};

    #[test]
    fn extracts_setf_callable_binding_name_target() {
        let tree = SyntaxTree::parse("(flet (((setf foo) (value object) value)) foo)\n")
            .expect("test input should parse");
        let view = tree
            .select_path(&Path::root_child(0))
            .expect("root form")
            .view();
        let bindings = view.children.get(1).expect("bindings list");
        let binding = bindings.children.first().expect("binding");

        let Some((name_view, name_path)) =
            callable_binding_name_target(binding, &Path::from_indexes(vec![]), 0)
        else {
            panic!("expected setf binding target");
        };

        assert_eq!(name_view.text.as_deref(), Some("foo"));
        assert_eq!(name_path.to_string(), "1.0.0.1");
    }

    #[test]
    fn collects_setf_callable_binding_name_rename() {
        let tree = SyntaxTree::parse("(flet (((setf foo) (value object) value)) foo)\n")
            .expect("test input should parse");
        let view = tree
            .select_path(&Path::root_child(0))
            .expect("root form")
            .view();
        let bindings = view.children.get(1).expect("bindings list");
        let binding = bindings.children.first().expect("binding");
        let from = crate::domain::sexpr::SymbolName::new("foo").expect("symbol");
        let to = crate::domain::sexpr::SymbolName::new("bar").expect("symbol");
        let context = TraversalContext {
            dialect: crate::domain::dialect::Dialect::CommonLisp,
            from: &from,
            to: &to,
            kind: LocalCallableRenameKind::Function,
        };
        let scopes = local_callable_scopes(
            MacroletRenameScope::default(),
            LocalCallableRenameKind::Function,
            CommonLispLocalCallableForm::Flet,
            TargetBindingPresence::Present,
        );
        let state = TraversalState {
            scope: scopes.binding_body,
            reader_lambda_body_scope: MacroletRenameScope::default(),
            quasiquote_depth: 0,
        };
        let mut renames = Vec::new();

        BindingTraversal::collect_binding_name_renames(
            binding,
            0,
            &Path::from_indexes(vec![]),
            crate::domain::common_lisp::CommonLispLocalCallableForm::Flet,
            context,
            state,
            &mut renames,
        );

        assert_eq!(renames.len(), 1);
        assert_eq!(renames[0].path, "1.0.0.1");
    }

    #[test]
    fn extracts_setf_callable_list_head_target() {
        let tree =
            SyntaxTree::parse("(flet (((setf foo) (value object) value)) ((setf foo) 1 thing))\n")
                .expect("test input should parse");
        let view = tree
            .select_path(&Path::root_child(0))
            .expect("root form")
            .view();
        let body = view.children.get(2).expect("body form");

        let (target, target_path) =
            callable_list_head_target(body, &Path::from_indexes(vec![0, 2]))
                .expect("setf callable head");

        assert_eq!(target.text.as_deref(), Some("foo"));
        assert_eq!(target_path.to_string(), "0.2.0.1");
    }
}
