use crate::application::usecase::rename::function::target::{
    CallableNameTarget, callable_name_target,
};
use crate::application::usecase::rename::macrolet::RenameFunctionOccurrence;
use crate::domain::sexpr::{ExpressionKind, ExpressionView, Path};

use super::super::super::reader::push_atom_rename_if_match;
use super::super::super::scope::MacroletRenameScope;
use super::super::state::{TraversalContext, TraversalState};

pub(super) fn collect_active_atom_rename(
    child: &ExpressionView,
    child_path: &Path,
    context: TraversalContext<'_>,
    state: TraversalState,
    scope: MacroletRenameScope,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    if child.kind == ExpressionKind::Atom && state.allows_active_rename(scope) {
        return push_atom_rename_if_match(child, child_path, context.from, context.to, renames);
    }
    false
}

pub(super) fn callable_binding_name_target<'a>(
    binding: &'a ExpressionView,
    path: &Path,
    binding_index: usize,
) -> Option<CallableNameTarget<'a>> {
    let name_view = binding.children.first()?;
    callable_name_target(name_view, &path.descendant([1, binding_index, 0]))
}

pub(super) fn callable_list_head_target<'a>(
    view: &'a ExpressionView,
    path: &Path,
) -> Option<CallableNameTarget<'a>> {
    let head = view.children.first()?;
    if head.kind != ExpressionKind::List {
        return None;
    }
    callable_name_target(head, &path.child(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::usecase::rename::macrolet::scope::{
        LocalCallableRenameKind, MacroletRenameScope,
    };
    use crate::application::usecase::rename::macrolet::traversal::BindingTraversal;
    use crate::application::usecase::rename::macrolet::traversal::core::RenameTraversalMode;
    use crate::application::usecase::rename::macrolet::traversal::state::TraversalState;
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

        let Some(target) = callable_binding_name_target(binding, &Path::from_indexes(vec![]), 0)
        else {
            panic!("expected setf binding target");
        };

        assert_eq!(target.text, "foo");
        assert_eq!(target.path.to_string(), "1.0.0.1");
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
        let state = TraversalState {
            scope: MacroletRenameScope::default(),
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

        let target = callable_list_head_target(body, &Path::from_indexes(vec![0, 2]))
            .expect("setf callable head");

        assert_eq!(target.text, "foo");
        assert_eq!(target.path.to_string(), "0.2.0.1");
    }
}
