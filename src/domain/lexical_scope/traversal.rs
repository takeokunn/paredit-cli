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
        "let" | "symbol-macrolet" => {
            collect_parallel_let_references(view, symbol, input, output);
            true
        }
        "let*" => {
            collect_sequential_let_references(view, symbol, input, output);
            true
        }
        "destructuring-bind" | "multiple-value-bind" => {
            collect_value_binding_references(view, symbol, input, output);
            true
        }
        "handler-case" | "restart-case" => {
            collect_clause_binding_references(view, symbol, input, output);
            true
        }
        "dolist" | "dotimes" => {
            collect_iteration_binding_references(view, symbol, input, output);
            true
        }
        "do" | "do*" => {
            collect_do_binding_references(view, symbol, input, output, head == "do*");
            true
        }
        "prog" | "prog*" => {
            collect_prog_binding_references(view, symbol, input, output, head == "prog*");
            true
        }
        "with-slots" | "with-accessors" => {
            collect_slot_binding_references(view, symbol, input, output);
            true
        }
        "lambda" | "fn" => parameter_form_binds(&view.children[1], symbol),
        "defun" | "defmacro" | "define-setf-expander" | "define-compiler-macro" => true,
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

    collect_unshadowed_symbol_references(value_form, symbol, input, output);

    if parameter_form_binds(binding_form, symbol) {
        return;
    }

    for body in &view.children[3..] {
        collect_unshadowed_symbol_references(body, symbol, input, output);
    }
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

    collect_unshadowed_symbol_references(protected_form, symbol, input, output);

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
        collect_unshadowed_symbol_references(clause, symbol, input, output);
        return;
    }

    let Some(parameter_form) = clause.children.get(1) else {
        return;
    };

    if parameter_form_binds(parameter_form, symbol) {
        return;
    }

    for body in &clause.children[2..] {
        collect_unshadowed_symbol_references(body, symbol, input, output);
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
        collect_unshadowed_symbol_references(source_form, symbol, input, output);
    }

    if iteration_binding_form_binds(binding_form, symbol) {
        return;
    }

    if let Some(result_form) = binding_form.children.get(2) {
        collect_unshadowed_symbol_references(result_form, symbol, input, output);
    }

    for body in &view.children[2..] {
        collect_unshadowed_symbol_references(body, symbol, input, output);
    }
}

fn collect_do_binding_references(
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
                collect_unshadowed_symbol_references(init_form, symbol, input, output);
            }
            if variable_spec_binds(spec, symbol) {
                return;
            }
        }
    } else {
        for spec in &binding_form.children {
            if let Some(init_form) = variable_spec_init_form(spec) {
                collect_unshadowed_symbol_references(init_form, symbol, input, output);
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
            collect_unshadowed_symbol_references(step_form, symbol, input, output);
        }
    }

    for body in &view.children[2..] {
        collect_unshadowed_symbol_references(body, symbol, input, output);
    }
}

fn collect_prog_binding_references(
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
                collect_unshadowed_symbol_references(init_form, symbol, input, output);
            }
            if variable_spec_binds(spec, symbol) {
                return;
            }
        }
    } else {
        for spec in &binding_form.children {
            if let Some(init_form) = variable_spec_init_form(spec) {
                collect_unshadowed_symbol_references(init_form, symbol, input, output);
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

    for body in &view.children[2..] {
        collect_unshadowed_symbol_references(body, symbol, input, output);
    }
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

    collect_unshadowed_symbol_references(instance_form, symbol, input, output);

    if slot_specs
        .children
        .iter()
        .any(|spec| slot_spec_binds(spec, symbol))
    {
        return;
    }

    for body in &view.children[3..] {
        collect_unshadowed_symbol_references(body, symbol, input, output);
    }
}

fn iteration_binding_form_binds(binding_form: &ExpressionView, symbol: &SymbolName) -> bool {
    binding_form
        .children
        .first()
        .and_then(atom_text)
        .is_some_and(|name| name == symbol.as_str())
}

fn slot_spec_binds(slot_spec: &ExpressionView, symbol: &SymbolName) -> bool {
    atom_text(slot_spec)
        .or_else(|| slot_spec.children.first().and_then(atom_text))
        .is_some_and(|name| name == symbol.as_str())
}

fn variable_spec_binds(spec: &ExpressionView, symbol: &SymbolName) -> bool {
    atom_text(spec)
        .or_else(|| spec.children.first().and_then(atom_text))
        .is_some_and(|name| name == symbol.as_str())
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
