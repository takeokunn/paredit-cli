use crate::domain::sexpr::{Delimiter, ExpressionView};

pub(super) fn binding_form_binds_name(view: &ExpressionView, name: &str) -> bool {
    let Some(head) = list_head(view) else {
        return false;
    };

    match head {
        "let" | "let*" | "symbol-macrolet" => view
            .children
            .get(1)
            .is_some_and(|bindings| binding_pairs_contain_name(bindings, name)),
        "lambda" => view
            .children
            .get(1)
            .is_some_and(|parameters| lambda_list_contains_name(parameters, name)),
        "fn" => view
            .children
            .get(1)
            .filter(|parameters| parameters.delimiter == Some(Delimiter::Bracket))
            .is_some_and(|parameters| pattern_contains_name(parameters, name)),
        "defun" | "defmacro" | "defn" | "define-setf-expander" | "define-compiler-macro" => view
            .children
            .get(2)
            .is_some_and(|parameters| lambda_list_contains_name(parameters, name)),
        "destructuring-bind" | "multiple-value-bind" => view
            .children
            .get(1)
            .is_some_and(|parameters| lambda_list_contains_name(parameters, name)),
        "dolist" | "dotimes" => view
            .children
            .get(1)
            .is_some_and(|binding_form| iteration_binding_binds_name(binding_form, name)),
        "do" | "do*" | "prog" | "prog*" => view
            .children
            .get(1)
            .is_some_and(|bindings| variable_specs_bind_name(bindings, name)),
        "with-slots" | "with-accessors" => view
            .children
            .get(1)
            .is_some_and(|slot_specs| slot_specs_bind_name(slot_specs, name)),
        _ => false,
    }
}

pub(super) fn child_shadowed_by_binding(
    view: &ExpressionView,
    name: &str,
    child_index: usize,
) -> bool {
    let Some(head) = list_head(view) else {
        return false;
    };

    match head {
        "let" | "let*" | "symbol-macrolet" => {
            child_index >= 2 && binding_form_binds_name(view, name)
        }
        "lambda" => child_index >= 2 && binding_form_binds_name(view, name),
        "fn" => child_index >= 2 && binding_form_binds_name(view, name),
        "defun" | "defmacro" | "defn" | "define-setf-expander" | "define-compiler-macro" => {
            child_index >= 3 && binding_form_binds_name(view, name)
        }
        "destructuring-bind" | "multiple-value-bind" => {
            child_index >= 3 && binding_form_binds_name(view, name)
        }
        "handler-case" | "restart-case" => view
            .children
            .get(child_index)
            .is_some_and(|clause| child_index >= 2 && clause_binds_name(clause, name)),
        "dolist" | "dotimes" => child_index >= 2 && binding_form_binds_name(view, name),
        "do" | "do*" | "prog" | "prog*" => child_index >= 2 && binding_form_binds_name(view, name),
        "with-slots" | "with-accessors" => child_index >= 3 && binding_form_binds_name(view, name),
        _ => false,
    }
}

pub(super) fn local_callable_bindings_child_index(view: &ExpressionView) -> Option<usize> {
    let head = list_head(view)?;
    matches!(head, "flet" | "labels" | "macrolet" | "compiler-macrolet").then_some(1)
}

pub(super) fn let_star_bindings_child_index(view: &ExpressionView) -> Option<usize> {
    matches!(list_head(view)?, "let*").then_some(1)
}

pub(super) fn iteration_bindings_child_index(view: &ExpressionView) -> Option<usize> {
    matches!(list_head(view)?, "dolist" | "dotimes").then_some(1)
}

pub(super) fn variable_bindings_child_index(view: &ExpressionView) -> Option<usize> {
    matches!(list_head(view)?, "do" | "do*" | "prog" | "prog*").then_some(1)
}

pub(super) fn variable_binding_form_is_sequential(view: &ExpressionView) -> bool {
    matches!(list_head(view), Some("do*" | "prog*"))
}

pub(super) fn variable_binding_form_has_step_forms(view: &ExpressionView) -> bool {
    matches!(list_head(view), Some("do" | "do*"))
}

pub(super) fn binding_pair_binds_name(binding: &ExpressionView, name: &str) -> bool {
    binding
        .children
        .first()
        .is_some_and(|pattern| pattern_contains_name(pattern, name))
}

pub(super) fn variable_spec_binds_name(binding: &ExpressionView, name: &str) -> bool {
    if atom_text(binding).is_some_and(|binding_name| binding_name == name) {
        return true;
    }

    binding
        .children
        .first()
        .is_some_and(|pattern| pattern_contains_name(pattern, name))
}

pub(super) fn iteration_binding_child_shadowed(
    binding_form: &ExpressionView,
    name: &str,
    child_index: usize,
) -> bool {
    child_index >= 2 && iteration_binding_binds_name(binding_form, name)
}

pub(super) fn local_callable_binding_child_shadowed(
    binding: &ExpressionView,
    name: &str,
    child_index: usize,
) -> bool {
    child_index >= 2
        && binding
            .children
            .get(1)
            .is_some_and(|parameters| lambda_list_contains_name(parameters, name))
}

fn binding_pairs_contain_name(bindings: &ExpressionView, name: &str) -> bool {
    if bindings.delimiter == Some(Delimiter::Bracket) {
        return bindings
            .children
            .iter()
            .step_by(2)
            .any(|binding| pattern_contains_name(binding, name));
    }

    bindings.children.iter().any(|binding| {
        binding
            .children
            .first()
            .is_some_and(|pattern| pattern_contains_name(pattern, name))
    })
}

fn clause_binds_name(clause: &ExpressionView, name: &str) -> bool {
    clause
        .children
        .get(1)
        .is_some_and(|parameters| lambda_list_contains_name(parameters, name))
}

fn iteration_binding_binds_name(binding_form: &ExpressionView, name: &str) -> bool {
    binding_form
        .children
        .first()
        .and_then(atom_text)
        .is_some_and(|binding_name| binding_name == name)
}

fn variable_specs_bind_name(bindings: &ExpressionView, name: &str) -> bool {
    bindings
        .children
        .iter()
        .any(|binding| variable_spec_binds_name(binding, name))
}

fn slot_specs_bind_name(slot_specs: &ExpressionView, name: &str) -> bool {
    slot_specs.children.iter().any(|slot_spec| {
        atom_text(slot_spec)
            .or_else(|| slot_spec.children.first().and_then(atom_text))
            .is_some_and(|binding_name| binding_name == name)
    })
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum LambdaListMode {
    Required,
    Optional,
    Key,
    Aux,
}

fn lambda_list_contains_name(parameter_form: &ExpressionView, name: &str) -> bool {
    let mut mode = LambdaListMode::Required;
    let mut index = 0usize;

    while index < parameter_form.children.len() {
        let child = &parameter_form.children[index];
        if let Some(marker) = atom_text(child) {
            match marker {
                "&optional" => {
                    mode = LambdaListMode::Optional;
                    index += 1;
                    continue;
                }
                "&key" => {
                    mode = LambdaListMode::Key;
                    index += 1;
                    continue;
                }
                "&aux" => {
                    mode = LambdaListMode::Aux;
                    index += 1;
                    continue;
                }
                "&rest" | "&body" | "&whole" | "&environment" => {
                    if parameter_form
                        .children
                        .get(index + 1)
                        .is_some_and(|next| pattern_contains_name(next, name))
                    {
                        return true;
                    }
                    index += 2;
                    continue;
                }
                "&allow-other-keys" => {
                    index += 1;
                    continue;
                }
                _ if marker.starts_with('&') => {
                    index += 1;
                    continue;
                }
                _ => {}
            }
        }

        if lambda_list_parameter_spec_contains_name(child, mode, name) {
            return true;
        }
        index += 1;
    }

    false
}

fn lambda_list_parameter_spec_contains_name(
    spec: &ExpressionView,
    mode: LambdaListMode,
    name: &str,
) -> bool {
    if atom_text(spec).is_some() || mode == LambdaListMode::Required {
        return pattern_contains_name(spec, name);
    }

    match mode {
        LambdaListMode::Required => pattern_contains_name(spec, name),
        LambdaListMode::Optional => {
            spec.children
                .first()
                .is_some_and(|parameter| pattern_contains_name(parameter, name))
                || supplied_p_contains_name(spec, name)
        }
        LambdaListMode::Key => {
            spec.children
                .first()
                .is_some_and(|parameter| key_parameter_contains_name(parameter, name))
                || supplied_p_contains_name(spec, name)
        }
        LambdaListMode::Aux => spec
            .children
            .first()
            .is_some_and(|parameter| pattern_contains_name(parameter, name)),
    }
}

fn key_parameter_contains_name(spec_name: &ExpressionView, name: &str) -> bool {
    if spec_name.children.len() >= 2
        && atom_text(&spec_name.children[0]).is_some_and(|designator| designator.starts_with(':'))
    {
        return pattern_contains_name(&spec_name.children[1], name);
    }

    pattern_contains_name(spec_name, name)
}

fn supplied_p_contains_name(spec: &ExpressionView, name: &str) -> bool {
    spec.children
        .get(2)
        .is_some_and(|supplied_p| pattern_contains_name(supplied_p, name))
}

fn pattern_contains_name(view: &ExpressionView, name: &str) -> bool {
    atom_text(view).map(|text| text == name).unwrap_or_else(|| {
        view.children
            .iter()
            .any(|child| pattern_contains_name(child, name))
    })
}

fn list_head(view: &ExpressionView) -> Option<&str> {
    view.children.first().and_then(atom_text)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    view.text.as_deref()
}
