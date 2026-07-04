use crate::domain::sexpr::{ExpressionKind, ExpressionView};

pub(super) fn is_package_head(head: &str, expected: &str) -> bool {
    head.rsplit(':')
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case(expected))
}

pub(super) fn package_option_name(head: &str) -> String {
    head.trim_start_matches(':').to_ascii_lowercase()
}

pub(super) fn package_option_atoms(option: &ExpressionView) -> impl Iterator<Item = String> + '_ {
    option
        .children
        .iter()
        .filter_map(atom_text)
        .map(ToOwned::to_owned)
}

pub(super) fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
