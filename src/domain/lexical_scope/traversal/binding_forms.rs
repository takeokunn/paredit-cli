use crate::domain::common_lisp::{
    CommonLispLocalCallableForm, CommonLispOperator, common_lisp_symbol_name_eq,
    local_callable_names,
};
use crate::domain::definition::definition_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::body::collect_body_forms;
use super::collect_unshadowed_symbol_references_in_context;
use super::lambda_lists::collect_lambda_list_references;
use crate::domain::lexical_scope::bindings::{
    binding_binds, generic_binding_groups, parameter_form_binds,
};

pub(super) fn collect_shadow_aware_special_form(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) -> bool {
    if view.kind != ExpressionKind::List || view.children.len() < 2 {
        return false;
    }

    let Some(head) = super::super::syntax::atom_text(&view.children[0]) else {
        return false;
    };

    let Some(operator) = CommonLispOperator::from_head(head) else {
        return false;
    };

    match operator {
        operator if operator.is_parallel_let_binding() => {
            collect_parallel_let_references(view, symbol, input, output);
            true
        }
        operator if operator.is_sequential_let_binding() => {
            collect_sequential_let_references(view, symbol, input, output);
            true
        }
        operator if operator.is_value_binding() => {
            collect_value_binding_references(view, symbol, input, output);
            true
        }
        operator if operator.is_clause_binding() => {
            collect_clause_binding_references(view, symbol, input, output);
            true
        }
        operator if operator.is_handler_bind_binding() => {
            collect_handler_bind_references(
                view,
                symbol,
                input,
                output,
                operator.includes_restart_bind_options(),
            );
            true
        }
        operator if operator.is_iteration_binding() => {
            collect_iteration_binding_references(view, symbol, input, output);
            true
        }
        operator if operator.is_do_binding() || operator.is_prog_binding() => {
            collect_do_like_binding_references(
                view,
                symbol,
                input,
                output,
                operator.is_sequential_variable_binding(),
            );
            true
        }
        operator if operator.is_slot_binding() => {
            collect_slot_binding_references(view, symbol, input, output);
            true
        }
        operator if operator.is_local_callable_binding() => {
            let Some(form) = operator.local_callable_form() else {
                return false;
            };
            collect_local_callable_references(view, symbol, input, output, form);
            true
        }
        CommonLispOperator::Locally => {
            collect_body_forms(&view.children[2..], symbol, input, output);
            true
        }
        operator if operator.is_lambda_like() => {
            view.children.get(1).is_some_and(|parameter_form| {
                collect_lambda_list_references(
                    parameter_form,
                    &view.children[2..],
                    symbol,
                    input,
                    output,
                )
            })
        }
        operator if operator.is_defun_like() => {
            let Some(shape) = definition_shape(Dialect::CommonLisp, view, head) else {
                return false;
            };

            if should_scan_definition_body(operator) {
                collect_body_forms(shape.body_forms(view), symbol, input, output);
            }
            true
        }
        _ => false,
    }
}

fn should_scan_definition_body(operator: CommonLispOperator) -> bool {
    !matches!(
        operator,
        CommonLispOperator::DefineSetfExpander | CommonLispOperator::DefineCompilerMacro
    )
}

fn collect_local_callable_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
    form: CommonLispLocalCallableForm,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };

    let local_names = local_callable_names(view);
    let body_is_shadowed = matches!(form, CommonLispLocalCallableForm::Labels)
        && local_names
            .iter()
            .any(|name| common_lisp_symbol_name_eq(name, symbol.as_str()));

    for spec in &binding_form.children {
        if spec.kind != ExpressionKind::List || spec.delimiter != Some(Delimiter::Paren) {
            continue;
        }

        let Some(parameter_form) = spec.children.get(1) else {
            continue;
        };

        let spec_body_forms: &[ExpressionView] = if body_is_shadowed {
            &[]
        } else {
            &spec.children[2..]
        };

        collect_lambda_list_references(parameter_form, spec_body_forms, symbol, input, output);
    }

    if local_names
        .iter()
        .any(|name| common_lisp_symbol_name_eq(name, symbol.as_str()))
    {
        return;
    }

    collect_body_forms(&view.children[2..], symbol, input, output);
}

fn collect_parallel_let_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };
    if binding_form.delimiter == Some(Delimiter::Bracket) {
        collect_sequential_let_references(view, symbol, input, output);
        return;
    }
    let Ok(bindings) = generic_binding_groups(binding_form) else {
        return;
    };

    for binding in &bindings {
        collect_unshadowed_symbol_references_in_context(&binding.value, symbol, input, output, 0);
    }

    if bindings
        .iter()
        .any(|binding| binding_binds(binding, symbol))
    {
        return;
    }

    collect_body_forms(&view.children[2..], symbol, input, output);
}

fn collect_sequential_let_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };
    let Ok(bindings) = generic_binding_groups(binding_form) else {
        return;
    };

    for binding in &bindings {
        collect_unshadowed_symbol_references_in_context(&binding.value, symbol, input, output, 0);
        if binding_binds(binding, symbol) {
            return;
        }
    }

    collect_body_forms(&view.children[2..], symbol, input, output);
}

fn collect_value_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };
    let Some(value_form) = view.children.get(2) else {
        return;
    };

    collect_unshadowed_symbol_references_in_context(value_form, symbol, input, output, 0);

    if parameter_form_binds(binding_form, symbol) {
        return;
    }

    collect_body_forms(&view.children[3..], symbol, input, output);
}

fn collect_clause_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    let Some(protected_form) = view.children.get(1) else {
        return;
    };

    collect_unshadowed_symbol_references_in_context(protected_form, symbol, input, output, 0);

    for clause in &view.children[2..] {
        collect_clause_body_references(clause, symbol, input, output);
    }
}

fn collect_clause_body_references(
    clause: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    if clause.kind != ExpressionKind::List {
        collect_unshadowed_symbol_references_in_context(clause, symbol, input, output, 0);
        return;
    }

    let Some(parameter_form) = clause.children.get(1) else {
        return;
    };

    if parameter_form_binds(parameter_form, symbol) {
        return;
    }

    collect_body_forms(&clause.children[2..], symbol, input, output);
}

fn collect_handler_bind_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
    include_restart_options: bool,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };

    for spec in &binding_form.children {
        if spec.kind != ExpressionKind::List || spec.delimiter != Some(Delimiter::Paren) {
            continue;
        }

        if let Some(function_form) = spec.children.get(1) {
            collect_unshadowed_symbol_references_in_context(
                function_form,
                symbol,
                input,
                output,
                0,
            );
        }

        if include_restart_options {
            collect_restart_bind_option_value_references(spec, symbol, input, output);
        }
    }

    collect_body_forms(&view.children[2..], symbol, input, output);
}

fn collect_restart_bind_option_value_references(
    spec: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    let mut index = 2;
    while index + 1 < spec.children.len() {
        collect_unshadowed_symbol_references_in_context(
            &spec.children[index + 1],
            symbol,
            input,
            output,
            0,
        );
        index += 2;
    }
}

fn collect_iteration_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };

    if let Some(source_form) = binding_form.children.get(1) {
        collect_unshadowed_symbol_references_in_context(source_form, symbol, input, output, 0);
    }

    if iteration_binding_form_binds(binding_form, symbol) {
        return;
    }

    if let Some(result_form) = binding_form.children.get(2) {
        collect_unshadowed_symbol_references_in_context(result_form, symbol, input, output, 0);
    }

    collect_body_forms(&view.children[2..], symbol, input, output);
}

fn collect_do_like_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
    sequential_scope: bool,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };

    if sequential_scope {
        for spec in &binding_form.children {
            if let Some(init_form) = variable_spec_init_form(spec) {
                collect_unshadowed_symbol_references_in_context(
                    init_form, symbol, input, output, 0,
                );
            }
            if variable_spec_binds(spec, symbol) {
                return;
            }
        }
    } else {
        for spec in &binding_form.children {
            if let Some(init_form) = variable_spec_init_form(spec) {
                collect_unshadowed_symbol_references_in_context(
                    init_form, symbol, input, output, 0,
                );
            }
        }
        if binding_form
            .children
            .iter()
            .any(|spec| variable_spec_binds(spec, symbol))
        {
            return;
        }
    }

    for spec in &binding_form.children {
        if let Some(step_form) = do_variable_spec_step_form(spec) {
            collect_unshadowed_symbol_references_in_context(step_form, symbol, input, output, 0);
        }
    }

    collect_body_forms(&view.children[2..], symbol, input, output);
}

fn collect_slot_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    let Some(slot_specs) = view.children.get(1) else {
        return;
    };
    let Some(instance_form) = view.children.get(2) else {
        return;
    };

    collect_unshadowed_symbol_references_in_context(instance_form, symbol, input, output, 0);

    if slot_specs
        .children
        .iter()
        .any(|spec| slot_spec_binds(spec, symbol))
    {
        return;
    }

    collect_body_forms(&view.children[3..], symbol, input, output);
}

fn iteration_binding_form_binds(binding_form: &ExpressionView, symbol: &SymbolName) -> bool {
    binding_form
        .children
        .first()
        .and_then(super::super::syntax::atom_text)
        .is_some_and(|name| common_lisp_symbol_name_eq(name, symbol.as_str()))
}

fn slot_spec_binds(slot_spec: &ExpressionView, symbol: &SymbolName) -> bool {
    super::super::syntax::atom_text(slot_spec)
        .or_else(|| {
            slot_spec
                .children
                .first()
                .and_then(super::super::syntax::atom_text)
        })
        .is_some_and(|name| common_lisp_symbol_name_eq(name, symbol.as_str()))
}

fn variable_spec_binds(spec: &ExpressionView, symbol: &SymbolName) -> bool {
    super::super::syntax::atom_text(spec)
        .or_else(|| {
            spec.children
                .first()
                .and_then(super::super::syntax::atom_text)
        })
        .is_some_and(|name| common_lisp_symbol_name_eq(name, symbol.as_str()))
}

fn variable_spec_init_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    (spec.kind == ExpressionKind::List)
        .then(|| spec.children.get(1))
        .flatten()
}

fn do_variable_spec_step_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    (spec.kind == ExpressionKind::List)
        .then(|| spec.children.get(2))
        .flatten()
}
