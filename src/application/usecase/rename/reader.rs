use crate::application::usecase::rename::selection::list_head;
use crate::domain::common_lisp::{
    common_lisp_operator_head_eq, normalize_common_lisp_operator_head,
};
use crate::domain::sexpr::reader::atom_text;
pub(crate) use crate::domain::sexpr::reader::{
    apply_reader_prefix_context, atom_symbol_span, atom_symbol_text,
};
use crate::domain::sexpr::{ExpressionKind, ExpressionView};

pub(crate) fn explicit_reader_form_kind(view: &ExpressionView) -> Option<String> {
    if view.kind != ExpressionKind::List || view.children.len() < 2 {
        return None;
    }

    let head = atom_text(&view.children[0])?;
    Some(normalize_common_lisp_operator_head(head).to_ascii_lowercase())
}

pub(crate) fn explicit_reader_function_lambda_view(
    view: &ExpressionView,
) -> Option<&ExpressionView> {
    if explicit_reader_form_kind(view)?.as_str() != "function" {
        return None;
    }

    let lambda_view = view.children.get(1)?;
    list_head(lambda_view)
        .is_some_and(|head| common_lisp_operator_head_eq(head, "lambda"))
        .then_some(lambda_view)
}

pub(crate) fn explicit_reader_function_lambda_body_children(
    view: &ExpressionView,
) -> Option<impl Iterator<Item = (usize, &ExpressionView)>> {
    explicit_reader_function_lambda_view(view)
        .map(|lambda_view| lambda_view.children.iter().enumerate().skip(2))
}
