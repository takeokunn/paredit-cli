use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::bindings::{binding_binds, generic_binding_groups, parameter_form_binds};
use super::syntax::atom_text;

pub fn collect_unshadowed_symbol_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    if atom_text(view).is_some_and(|text| text == symbol.as_str()) {
        output.push(view.span);
        return;
    }

    if collect_shadow_aware_special_form(view, symbol, input, output) {
        return;
    }

    for child in &view.children {
        collect_unshadowed_symbol_references(child, symbol, input, output);
    }
}

fn collect_shadow_aware_special_form(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) -> bool {
    if view.kind != ExpressionKind::List || view.children.len() < 2 {
        return false;
    }

    let Some(head) = atom_text(&view.children[0]) else {
        return false;
    };

    match head {
        "let" => {
            collect_parallel_let_references(view, symbol, input, output);
            true
        }
        "let*" => {
            collect_sequential_let_references(view, symbol, input, output);
            true
        }
        "lambda" | "fn" => parameter_form_binds(&view.children[1], symbol),
        "defun" | "defmacro" => true,
        _ => false,
    }
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
        collect_unshadowed_symbol_references(&binding.value, symbol, input, output);
    }

    if bindings
        .iter()
        .any(|binding| binding_binds(binding, symbol))
    {
        return;
    }

    for body in &view.children[2..] {
        collect_unshadowed_symbol_references(body, symbol, input, output);
    }
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
        collect_unshadowed_symbol_references(&binding.value, symbol, input, output);
        if binding_binds(binding, symbol) {
            return;
        }
    }

    for body in &view.children[2..] {
        collect_unshadowed_symbol_references(body, symbol, input, output);
    }
}
