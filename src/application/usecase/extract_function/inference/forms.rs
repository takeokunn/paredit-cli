use crate::application::usecase::callable_scope::common_lisp_local_callable_form;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

use super::super::syntax::{atom_text, list_head};
use super::bindings::extract_function_binding_entries;
use super::patterns::parameter_names;
use super::symbols::is_extract_function_param_candidate;

pub(super) fn collect_inferred_extract_function_special_form(
    dialect: Dialect,
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

    if common_lisp_local_callable_form(dialect, head).is_some() {
        return collect_inferred_extract_function_local_callable_form(
            view,
            explicit_params,
            bound_params,
            params,
        );
    }

    match head {
        "let" | "symbol-macrolet" => collect_inferred_extract_function_let(
            dialect,
            view,
            explicit_params,
            bound_params,
            params,
        ),
        "let*" => collect_inferred_extract_function_let_star(
            dialect,
            view,
            explicit_params,
            bound_params,
            params,
        ),
        "destructuring-bind" | "multiple-value-bind" => {
            collect_inferred_extract_function_value_binding(
                dialect,
                view,
                explicit_params,
                bound_params,
                params,
            )
        }
        "handler-case" | "restart-case" => collect_inferred_extract_function_clause_form(
            dialect,
            view,
            explicit_params,
            bound_params,
            params,
        ),
        "dolist" | "dotimes" => collect_inferred_extract_function_iteration_binding(
            dialect,
            view,
            explicit_params,
            bound_params,
            params,
        ),
        "do" | "do*" => collect_inferred_extract_function_do(
            dialect,
            view,
            explicit_params,
            bound_params,
            params,
            head == "do*",
        ),
        "prog" | "prog*" => collect_inferred_extract_function_prog(
            dialect,
            view,
            explicit_params,
            bound_params,
            params,
            head == "prog*",
        ),
        "with-slots" | "with-accessors" => collect_inferred_extract_function_slot_binding(
            dialect,
            view,
            explicit_params,
            bound_params,
            params,
        ),
        "lambda" | "fn" => collect_inferred_extract_function_lambda(
            dialect,
            view,
            1,
            explicit_params,
            bound_params,
            params,
        ),
        "defun" | "defmacro" | "define-setf-expander" | "define-compiler-macro" => {
            collect_inferred_extract_function_lambda(
                dialect,
                view,
                2,
                explicit_params,
                bound_params,
                params,
            )
        }
        _ => false,
    }
}

fn collect_inferred_extract_function_let(
    dialect: Dialect,
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
            dialect,
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
            dialect,
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
            dialect,
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
    dialect: Dialect,
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
            dialect,
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
            dialect,
            body,
            false,
            explicit_params,
            &current_bound_params,
            params,
        );
    }
    true
}

fn collect_inferred_extract_function_value_binding(
    dialect: Dialect,
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) -> bool {
    let Some(binding_form) = view.children.get(1) else {
        return false;
    };
    let Some(value_form) = view.children.get(2) else {
        return false;
    };

    super::collect_inferred_extract_function_params(
        dialect,
        value_form,
        false,
        explicit_params,
        bound_params,
        params,
    );

    let names = parameter_names(binding_form);
    let body_bound_params =
        extend_extract_function_bound_params(bound_params, names.iter().map(String::as_str));
    for body in &view.children[3..] {
        super::collect_inferred_extract_function_params(
            dialect,
            body,
            false,
            explicit_params,
            &body_bound_params,
            params,
        );
    }
    true
}

fn collect_inferred_extract_function_clause_form(
    dialect: Dialect,
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) -> bool {
    let Some(protected_form) = view.children.get(1) else {
        return false;
    };

    super::collect_inferred_extract_function_params(
        dialect,
        protected_form,
        false,
        explicit_params,
        bound_params,
        params,
    );

    for clause in &view.children[2..] {
        if clause.kind != ExpressionKind::List || clause.delimiter != Some(Delimiter::Paren) {
            super::collect_inferred_extract_function_params(
                dialect,
                clause,
                false,
                explicit_params,
                bound_params,
                params,
            );
            continue;
        }

        let Some(parameter_form) = clause.children.get(1) else {
            continue;
        };
        let names = parameter_names(parameter_form);
        let clause_bound_params =
            extend_extract_function_bound_params(bound_params, names.iter().map(String::as_str));
        for body in clause.children.iter().skip(2) {
            super::collect_inferred_extract_function_params(
                dialect,
                body,
                false,
                explicit_params,
                &clause_bound_params,
                params,
            );
        }
    }
    true
}

fn collect_inferred_extract_function_iteration_binding(
    dialect: Dialect,
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) -> bool {
    let Some(binding_form) = view.children.get(1) else {
        return false;
    };

    if let Some(source_form) = binding_form.children.get(1) {
        super::collect_inferred_extract_function_params(
            dialect,
            source_form,
            false,
            explicit_params,
            bound_params,
            params,
        );
    }

    let body_bound_params = extend_extract_function_bound_params(
        bound_params,
        binding_form
            .children
            .first()
            .and_then(atom_text)
            .into_iter(),
    );

    if let Some(result_form) = binding_form.children.get(2) {
        super::collect_inferred_extract_function_params(
            dialect,
            result_form,
            false,
            explicit_params,
            &body_bound_params,
            params,
        );
    }

    for body in &view.children[2..] {
        super::collect_inferred_extract_function_params(
            dialect,
            body,
            false,
            explicit_params,
            &body_bound_params,
            params,
        );
    }
    true
}

fn collect_inferred_extract_function_do(
    dialect: Dialect,
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
    sequential_scope: bool,
) -> bool {
    let Some(binding_form) = view.children.get(1) else {
        return false;
    };

    let mut body_bound_params = bound_params.to_vec();
    if sequential_scope {
        for spec in &binding_form.children {
            if let Some(init_form) = iteration_spec_init_form(spec) {
                super::collect_inferred_extract_function_params(
                    dialect,
                    init_form,
                    false,
                    explicit_params,
                    &body_bound_params,
                    params,
                );
            }
            if let Some(name) = iteration_spec_bound_name(spec) {
                push_extract_function_bound_param(&mut body_bound_params, name);
            }
        }
    } else {
        for spec in &binding_form.children {
            if let Some(init_form) = iteration_spec_init_form(spec) {
                super::collect_inferred_extract_function_params(
                    dialect,
                    init_form,
                    false,
                    explicit_params,
                    bound_params,
                    params,
                );
            }
        }
        body_bound_params = extend_extract_function_bound_params(
            bound_params,
            binding_form
                .children
                .iter()
                .filter_map(iteration_spec_bound_name),
        );
    }

    for spec in &binding_form.children {
        if let Some(step_form) = iteration_spec_step_form(spec) {
            super::collect_inferred_extract_function_params(
                dialect,
                step_form,
                false,
                explicit_params,
                &body_bound_params,
                params,
            );
        }
    }

    for body in &view.children[2..] {
        super::collect_inferred_extract_function_params(
            dialect,
            body,
            false,
            explicit_params,
            &body_bound_params,
            params,
        );
    }
    true
}

fn collect_inferred_extract_function_prog(
    dialect: Dialect,
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
    sequential_scope: bool,
) -> bool {
    let Some(binding_form) = view.children.get(1) else {
        return false;
    };

    let mut body_bound_params = bound_params.to_vec();
    if sequential_scope {
        for spec in &binding_form.children {
            if let Some(init_form) = iteration_spec_init_form(spec) {
                super::collect_inferred_extract_function_params(
                    dialect,
                    init_form,
                    false,
                    explicit_params,
                    &body_bound_params,
                    params,
                );
            }
            if let Some(name) = iteration_spec_bound_name(spec) {
                push_extract_function_bound_param(&mut body_bound_params, name);
            }
        }
    } else {
        for spec in &binding_form.children {
            if let Some(init_form) = iteration_spec_init_form(spec) {
                super::collect_inferred_extract_function_params(
                    dialect,
                    init_form,
                    false,
                    explicit_params,
                    bound_params,
                    params,
                );
            }
        }
        body_bound_params = extend_extract_function_bound_params(
            bound_params,
            binding_form
                .children
                .iter()
                .filter_map(iteration_spec_bound_name),
        );
    }

    for body in &view.children[2..] {
        super::collect_inferred_extract_function_params(
            dialect,
            body,
            false,
            explicit_params,
            &body_bound_params,
            params,
        );
    }
    true
}

fn collect_inferred_extract_function_slot_binding(
    dialect: Dialect,
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) -> bool {
    let Some(slot_specs) = view.children.get(1) else {
        return false;
    };
    let Some(instance_form) = view.children.get(2) else {
        return false;
    };

    super::collect_inferred_extract_function_params(
        dialect,
        instance_form,
        false,
        explicit_params,
        bound_params,
        params,
    );

    let body_bound_params = extend_extract_function_bound_params(
        bound_params,
        slot_specs.children.iter().filter_map(slot_spec_bound_name),
    );
    for body in &view.children[3..] {
        super::collect_inferred_extract_function_params(
            dialect,
            body,
            false,
            explicit_params,
            &body_bound_params,
            params,
        );
    }
    true
}

fn slot_spec_bound_name(slot_spec: &ExpressionView) -> Option<&str> {
    atom_text(slot_spec).or_else(|| slot_spec.children.first().and_then(atom_text))
}

fn iteration_spec_bound_name(spec: &ExpressionView) -> Option<&str> {
    atom_text(spec).or_else(|| spec.children.first().and_then(atom_text))
}

fn iteration_spec_init_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    (spec.kind == ExpressionKind::List)
        .then(|| spec.children.get(1))
        .flatten()
}

fn iteration_spec_step_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    (spec.kind == ExpressionKind::List)
        .then(|| spec.children.get(2))
        .flatten()
}

fn collect_inferred_extract_function_lambda(
    dialect: Dialect,
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
            dialect,
            body,
            false,
            explicit_params,
            &body_bound_params,
            params,
        );
    }
    true
}

fn collect_inferred_extract_function_local_callable_form(
    view: &ExpressionView,
    explicit_params: &[String],
    bound_params: &[String],
    params: &mut Vec<String>,
) -> bool {
    let Some(binding_form) = view.children.get(1) else {
        return false;
    };
    if binding_form.delimiter != Some(Delimiter::Paren) {
        return false;
    }

    for binding in &binding_form.children {
        if binding.kind != ExpressionKind::List || binding.delimiter != Some(Delimiter::Paren) {
            continue;
        }
        let Some(parameter_form) = binding.children.get(1) else {
            continue;
        };
        let lambda_bound_params = extend_extract_function_bound_params(
            bound_params,
            parameter_names(parameter_form).iter().map(String::as_str),
        );
        for body in binding.children.iter().skip(2) {
            super::collect_inferred_extract_function_params(
                Dialect::CommonLisp,
                body,
                false,
                explicit_params,
                &lambda_bound_params,
                params,
            );
        }
    }

    for body in &view.children[2..] {
        super::collect_inferred_extract_function_params(
            Dialect::CommonLisp,
            body,
            false,
            explicit_params,
            bound_params,
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
