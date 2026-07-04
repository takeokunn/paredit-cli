mod bindings;
mod forms;
mod patterns;
mod symbols;

use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

use super::syntax::atom_text;
use forms::collect_inferred_extract_function_special_form;
use symbols::is_extract_function_param_candidate;

pub(super) fn infer_extract_function_params(
    selection: &ExpressionView,
    explicit_params: &[String],
) -> Vec<String> {
    let mut params = Vec::new();
    collect_inferred_extract_function_params(
        selection,
        false,
        explicit_params,
        &Vec::new(),
        &mut params,
    );
    params
}

fn collect_inferred_extract_function_params(
    view: &ExpressionView,
    is_call_head: bool,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) {
    if let Some(text) = atom_text(view) {
        if !is_call_head
            && is_extract_function_param_candidate(text)
            && !explicit_params.iter().any(|param| param == text)
            && !bound_params.iter().any(|param| param == text)
            && !params.iter().any(|param| param == text)
        {
            params.push(text.to_owned());
        }
        return;
    }

    if collect_inferred_extract_function_special_form(view, explicit_params, bound_params, params) {
        return;
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_inferred_extract_function_params(
            child,
            view.kind == ExpressionKind::List
                && view.delimiter == Some(Delimiter::Paren)
                && index == 0,
            explicit_params,
            bound_params,
            params,
        );
    }
}
