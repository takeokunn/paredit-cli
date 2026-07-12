use crate::domain::common_lisp::CommonLispOperator;
use crate::domain::definition::definition_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, SymbolName};

use super::body::collect_body_forms;
use super::collect_unshadowed_symbol_references_in_context;
use super::lambda_lists::collect_lambda_list_references;
use crate::domain::lexical_scope::bindings::{binding_binds, generic_binding_groups};

mod clause_bindings;
mod local_callables;
mod loop_bindings;
mod slots;
mod value_bindings;

use clause_bindings::{collect_clause_binding_references, collect_handler_bind_references};
use local_callables::collect_local_callable_references;
use loop_bindings::{
    collect_do_like_binding_references, collect_iteration_binding_references,
    collect_parallel_let_references, collect_sequential_let_references,
};
use slots::collect_slot_binding_references;
use value_bindings::collect_value_binding_references;

pub(super) fn collect_shadow_aware_special_form(
    dialect: Dialect,
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

    if dialect == Dialect::Scheme && is_scheme_named_let(view, head) {
        collect_named_let_references(dialect, view, symbol, input, output);
        return true;
    }

    let Some(operator) = CommonLispOperator::from_head(head) else {
        return false;
    };

    match operator {
        operator if operator.is_parallel_let_binding() => {
            collect_parallel_let_references(dialect, view, symbol, input, output);
            true
        }
        operator if operator.is_sequential_let_binding() => {
            collect_sequential_let_references(dialect, view, symbol, input, output);
            true
        }
        operator if operator.is_value_binding() => {
            collect_value_binding_references(dialect, view, symbol, input, output);
            true
        }
        operator if operator.is_clause_binding() => {
            collect_clause_binding_references(dialect, view, symbol, input, output);
            true
        }
        operator if operator.is_handler_bind_binding() => {
            collect_handler_bind_references(
                dialect,
                view,
                symbol,
                input,
                output,
                operator.includes_restart_bind_options(),
            );
            true
        }
        operator if operator.is_iteration_binding() => {
            collect_iteration_binding_references(dialect, view, symbol, input, output);
            true
        }
        operator if operator.is_do_binding() || operator.is_prog_binding() => {
            collect_do_like_binding_references(
                dialect,
                view,
                symbol,
                input,
                output,
                operator.is_sequential_variable_binding(),
            );
            true
        }
        operator if operator.is_slot_binding() => {
            collect_slot_binding_references(dialect, view, symbol, input, output);
            true
        }
        operator if operator.is_local_callable_binding() => {
            let Some(form) = operator.local_callable_form() else {
                return false;
            };
            collect_local_callable_references(dialect, view, symbol, input, output, form);
            true
        }
        CommonLispOperator::Locally => {
            collect_body_forms(dialect, &view.children[2..], symbol, input, output);
            true
        }
        operator if operator.is_lambda_like() => {
            view.children.get(1).is_some_and(|parameter_form| {
                collect_lambda_list_references(
                    dialect,
                    parameter_form,
                    &view.children[2..],
                    symbol,
                    input,
                    output,
                )
            })
        }
        operator if operator.is_defun_like() => {
            let Some(shape) = definition_shape(dialect, view, head) else {
                return false;
            };

            let body_forms: &[ExpressionView] = if should_scan_definition_body(operator) {
                shape.body_forms(view)
            } else {
                &[]
            };

            match shape.lambda_list(view) {
                // A lambda-list parameter's default-value form (`&optional
                // (y *default*)`) is an ordinary evaluated expression, not a
                // binding, and must be scanned like any other reference; only
                // the parameter *names* shadow the body. COLLECT_LAMBDA_LIST_
                // REFERENCES handles both in one pass, matching the LAMBDA/
                // FLET-bound-function branch above; the plain body-only scan
                // this replaced skipped every default-value form entirely.
                Some(parameter_form) => {
                    collect_lambda_list_references(
                        dialect,
                        parameter_form,
                        body_forms,
                        symbol,
                        input,
                        output,
                    );
                }
                None => {
                    collect_body_forms(dialect, body_forms, symbol, input, output);
                }
            }
            true
        }
        _ => false,
    }
}

fn is_scheme_named_let(view: &ExpressionView, head: &str) -> bool {
    (head == "let" || head == "let*")
        && view
            .children
            .get(1)
            .and_then(super::super::syntax::atom_text)
            .is_some()
}

fn collect_named_let_references(
    dialect: Dialect,
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    let Some(loop_name) = view.children.get(1) else {
        return;
    };
    let Some(binding_form) = view.children.get(2) else {
        return;
    };

    let bindings = generic_binding_groups(binding_form).unwrap_or_default();
    for binding in &bindings {
        if let Some(value) = &binding.value {
            collect_unshadowed_symbol_references_in_context(
                dialect, value, symbol, input, output, 0,
            );
        }
    }

    let loop_name_binds = super::super::syntax::atom_text(loop_name)
        .is_some_and(|name| super::symbol_name_matches(dialect, name, symbol.as_str()));
    if loop_name_binds
        || bindings
            .iter()
            .any(|binding| binding_binds(binding, symbol))
    {
        return;
    }

    collect_body_forms(dialect, &view.children[3..], symbol, input, output);
}

fn should_scan_definition_body(operator: CommonLispOperator) -> bool {
    !matches!(
        operator,
        CommonLispOperator::DefineSetfExpander | CommonLispOperator::DefineCompilerMacro
    )
}
