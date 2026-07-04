use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

use super::super::syntax::list_head;
use super::bindings::extract_function_binding_entries;
use super::patterns::parameter_names;
use super::symbols::is_extract_function_param_candidate;

pub(super) fn collect_inferred_extract_function_special_form(
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) -> bool {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return false;
    }

    let Some(head) = list_head(view) else {
        return false;
    };

    match head {
        "let" => collect_inferred_extract_function_let(view, explicit_params, bound_params, params),
        "let*" => {
            collect_inferred_extract_function_let_star(view, explicit_params, bound_params, params)
        }
        "lambda" | "fn" => {
            collect_inferred_extract_function_lambda(view, 1, explicit_params, bound_params, params)
        }
        "defun" | "defmacro" => {
            collect_inferred_extract_function_lambda(view, 2, explicit_params, bound_params, params)
        }
        _ => false,
    }
}

fn collect_inferred_extract_function_let(
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) -> bool {
    let Some(binding_form) = view.children.get(1) else {
        return false;
    };
    if binding_form.delimiter == Some(Delimiter::Bracket) {
        return collect_inferred_extract_function_let_star(
            view,
            explicit_params,
            bound_params,
            params,
        );
    }
    let Some(bindings) = extract_function_binding_entries(binding_form) else {
        return false;
    };

    for binding in &bindings {
        super::collect_inferred_extract_function_params(
            &binding.value,
            false,
            explicit_params,
            bound_params,
            params,
        );
    }

    let body_bound_params = extend_extract_function_bound_params(
        bound_params,
        bindings
            .iter()
            .flat_map(|binding| binding.names.iter().map(String::as_str)),
    );
    for body in &view.children[2..] {
        super::collect_inferred_extract_function_params(
            body,
            false,
            explicit_params,
            &body_bound_params,
            params,
        );
    }
    true
}

fn collect_inferred_extract_function_let_star(
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) -> bool {
    let Some(binding_form) = view.children.get(1) else {
        return false;
    };
    let Some(bindings) = extract_function_binding_entries(binding_form) else {
        return false;
    };

    let mut current_bound_params = bound_params.to_vec();
    for binding in &bindings {
        super::collect_inferred_extract_function_params(
            &binding.value,
            false,
            explicit_params,
            &current_bound_params,
            params,
        );
        for name in &binding.names {
            push_extract_function_bound_param(&mut current_bound_params, name);
        }
    }

    for body in &view.children[2..] {
        super::collect_inferred_extract_function_params(
            body,
            false,
            explicit_params,
            &current_bound_params,
            params,
        );
    }
    true
}

fn collect_inferred_extract_function_lambda(
    view: &ExpressionView,
    parameter_index: usize,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) -> bool {
    let Some(parameter_form) = view.children.get(parameter_index) else {
        return false;
    };
    let names = parameter_names(parameter_form);
    let body_bound_params =
        extend_extract_function_bound_params(bound_params, names.iter().map(String::as_str));
    for body in &view.children[parameter_index + 1..] {
        super::collect_inferred_extract_function_params(
            body,
            false,
            explicit_params,
            &body_bound_params,
            params,
        );
    }
    true
}

fn extend_extract_function_bound_params<'a>(
    bound_params: &[String],
    names: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    let mut extended = bound_params.to_vec();
    for name in names {
        push_extract_function_bound_param(&mut extended, name);
    }
    extended
}

fn push_extract_function_bound_param(bound_params: &mut Vec<String>, name: &str) {
    if is_extract_function_param_candidate(name) && !bound_params.iter().any(|param| param == name)
    {
        bound_params.push(name.to_owned());
    }
}
