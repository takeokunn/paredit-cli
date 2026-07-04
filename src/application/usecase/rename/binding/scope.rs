use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::super::selection::atom_text;
use super::forms::{binding_binds, generic_binding_groups, parameter_form_binds};

pub(super) fn collect_symbol_atom_spans_unshadowed(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    if atom_text(view).is_some_and(|text| text == symbol.as_str()) {
        output.push(view.span);
        return;
    }

    if collect_shadow_aware_special_form(view, symbol, output, shadowed_scope_count, input) {
        return;
    }

    for child in &view.children {
        collect_symbol_atom_spans_unshadowed(child, symbol, output, shadowed_scope_count, input);
    }
}

fn collect_shadow_aware_special_form(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) -> bool {
    if view.kind != ExpressionKind::List || view.children.len() < 2 {
        return false;
    }

    let Some(head) = atom_text(&view.children[0]) else {
        return false;
    };

    match head {
        "let" | "symbol-macrolet" => {
            collect_parallel_let_references(view, symbol, output, shadowed_scope_count, input);
            true
        }
        "let*" => {
            collect_sequential_let_references(view, symbol, output, shadowed_scope_count, input);
            true
        }
        "destructuring-bind" | "multiple-value-bind" => {
            collect_value_binding_references(view, symbol, output, shadowed_scope_count, input);
            true
        }
        "lambda" | "fn" => {
            if parameter_form_binds(&view.children[1], symbol, input) {
                *shadowed_scope_count += 1;
                return true;
            }
            false
        }
        "defun" | "defmacro" | "define-setf-expander" | "define-compiler-macro" => {
            if view.children.len() > 2 && parameter_form_binds(&view.children[2], symbol, input) {
                *shadowed_scope_count += 1;
            }
            true
        }
        _ => false,
    }
}

fn collect_parallel_let_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };
    if binding_form.delimiter == Some(Delimiter::Bracket) {
        collect_sequential_let_references(view, symbol, output, shadowed_scope_count, input);
        return;
    }
    let Ok(bindings) = generic_binding_groups(binding_form, input) else {
        return;
    };

    for binding in &bindings {
        collect_symbol_atom_spans_unshadowed(
            &binding.value,
            symbol,
            output,
            shadowed_scope_count,
            input,
        );
    }

    if bindings
        .iter()
        .any(|binding| binding_binds(binding, symbol))
    {
        *shadowed_scope_count += 1;
        return;
    }

    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
    }
}

fn collect_value_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    if let Some(value_form) = view.children.get(2) {
        collect_symbol_atom_spans_unshadowed(
            value_form,
            symbol,
            output,
            shadowed_scope_count,
            input,
        );
    }

    if parameter_form_binds(&view.children[1], symbol, input) {
        *shadowed_scope_count += 1;
        return;
    }

    for body in &view.children[3..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
    }
}

fn collect_sequential_let_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };
    let Ok(bindings) = generic_binding_groups(binding_form, input) else {
        return;
    };

    for binding in &bindings {
        collect_symbol_atom_spans_unshadowed(
            &binding.value,
            symbol,
            output,
            shadowed_scope_count,
            input,
        );
        if binding_binds(binding, symbol) {
            *shadowed_scope_count += 1;
            return;
        }
    }

    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
    }
}
