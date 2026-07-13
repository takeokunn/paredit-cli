mod bindings;
mod forms;
mod patterns;
mod symbols;

use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::common_lisp::is_common_lisp_declaration_form;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

use super::syntax::{atom_text, list_head};
use forms::collect_inferred_extract_function_special_form;
use symbols::is_extract_function_param_candidate;

pub(super) fn infer_extract_function_params(
    dialect: Dialect,
    selection: &ExpressionView,
    explicit_params: &[String],
) -> Vec<String> {
    let mut params = Vec::new();
    collect_inferred_extract_function_params(
        dialect,
        selection,
        false,
        explicit_params,
        &Vec::new(),
        &mut params,
    );
    params
}

pub(super) fn extract_function_param_name_eq(dialect: Dialect, left: &str, right: &str) -> bool {
    match dialect {
        Dialect::CommonLisp | Dialect::Unknown => common_lisp_symbol_reference_eq(left, right),
        _ => left == right,
    }
}

fn collect_inferred_extract_function_params(
    dialect: Dialect,
    view: &ExpressionView,
    is_call_head: bool,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) {
    if let Some(text) = atom_text(view) {
        if !is_call_head
            && is_extract_function_param_candidate(text)
            && !explicit_params
                .iter()
                .any(|param| extract_function_param_name_eq(dialect, param, text))
            && !bound_params
                .iter()
                .any(|param| extract_function_param_name_eq(dialect, param, text))
            && !params
                .iter()
                .any(|param| extract_function_param_name_eq(dialect, param, text))
        {
            params.push(text.to_owned());
        }
        return;
    }

    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        if let Some(head) = list_head(view) {
            if is_common_lisp_declaration_form(head) {
                return;
            }
        }
    }

    if collect_inferred_extract_function_special_form(
        dialect,
        view,
        explicit_params,
        bound_params,
        params,
    ) {
        return;
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_inferred_extract_function_params(
            dialect,
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
