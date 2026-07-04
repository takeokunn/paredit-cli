use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionKind, ExpressionView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalCallableForm {
    Flet,
    Labels,
    Macrolet,
    CompilerMacrolet,
}

pub(crate) fn common_lisp_local_callable_form(
    dialect: Dialect,
    head: &str,
) -> Option<LocalCallableForm> {
    if !matches!(dialect, Dialect::CommonLisp | Dialect::Unknown) {
        return None;
    }
    match head {
        "flet" => Some(LocalCallableForm::Flet),
        "labels" => Some(LocalCallableForm::Labels),
        "macrolet" => Some(LocalCallableForm::Macrolet),
        "compiler-macrolet" => Some(LocalCallableForm::CompilerMacrolet),
        _ => None,
    }
}

pub(crate) fn is_macro_callable_form(form: LocalCallableForm) -> bool {
    matches!(
        form,
        LocalCallableForm::Macrolet | LocalCallableForm::CompilerMacrolet
    )
}

pub(crate) fn local_callable_names(view: &ExpressionView) -> Vec<String> {
    view.children
        .get(1)
        .into_iter()
        .flat_map(|bindings| bindings.children.iter())
        .filter_map(local_callable_name)
        .map(ToOwned::to_owned)
        .collect()
}

pub(crate) fn is_local_callable_bound(scope: &[String], head: &str) -> bool {
    scope.iter().any(|name| name == head)
}

fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

fn local_callable_name(binding: &ExpressionView) -> Option<&str> {
    if binding.kind != ExpressionKind::List {
        return None;
    }
    atom_child(binding, 0)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
