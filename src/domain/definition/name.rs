use crate::domain::common_lisp::common_lisp_operator_head_eq;
use crate::domain::sexpr::{ExpressionKind, ExpressionView, Path};

use super::DefinitionNameTarget;

pub(super) fn definition_name_text(view: &ExpressionView) -> Option<&str> {
    atom_text(view).or_else(|| setf_callable_name_text(view))
}

pub(super) fn definition_name_target<'a>(
    view: &'a ExpressionView,
    path: &Path,
) -> Option<DefinitionNameTarget<'a>> {
    if let Some(text) = atom_text(view) {
        return Some(DefinitionNameTarget {
            path: path.clone(),
            span: view.span,
            text,
        });
    }

    let name = setf_callable_name_view(view)?;
    Some(DefinitionNameTarget {
        path: path.child(1),
        span: name.span,
        text: atom_text(name)?,
    })
}

fn setf_callable_name_text(view: &ExpressionView) -> Option<&str> {
    setf_callable_name_view(view).and_then(atom_text)
}

fn setf_callable_name_view(view: &ExpressionView) -> Option<&ExpressionView> {
    (view.kind == ExpressionKind::List).then_some(())?;
    let head = view.children.first().and_then(atom_text)?;
    common_lisp_operator_head_eq(head, "setf").then_some(())?;
    view.children
        .get(1)
        .filter(|name| name.kind == ExpressionKind::Atom)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
