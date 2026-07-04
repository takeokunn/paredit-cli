use crate::domain::sexpr::{ExpressionKind, ExpressionView};

pub(super) fn list_child(view: &ExpressionView, index: usize) -> Option<&ExpressionView> {
    (view.kind == ExpressionKind::List)
        .then_some(())
        .and_then(|_| view.children.get(index))
}

pub(super) fn count_lambda_parameters(lambda_list: &ExpressionView) -> usize {
    lambda_list
        .children
        .iter()
        .filter(|child| {
            atom_text(child)
                .map(|text| !text.starts_with('&'))
                .unwrap_or(false)
        })
        .count()
}

pub(super) fn list_head(view: &ExpressionView) -> Option<&str> {
    atom_child(view, 0)
}

pub(super) fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    list_child(view, index).and_then(atom_text)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    if view.kind == ExpressionKind::Atom {
        view.text.as_deref()
    } else {
        None
    }
}
