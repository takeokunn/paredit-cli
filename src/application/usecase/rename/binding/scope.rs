use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::super::selection::atom_text;
use super::destructure::binding_pattern_name_spans;
use super::forms::{
    binding_binds, generic_binding_groups, parameter_form_binds, specialized_parameter_form_binds,
};

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
        "defmethod" | "cl-defmethod" => {
            collect_defmethod_references(view, symbol, output, shadowed_scope_count, input);
            true
        }
        "defun" | "defmacro" | "define-setf-expander" | "define-compiler-macro" => {
            if view.children.len() > 2 && parameter_form_binds(&view.children[2], symbol, input) {
                *shadowed_scope_count += 1;
            }
            true
        }
        "handler-case" | "restart-case" => {
            collect_clause_form_references(view, symbol, output, shadowed_scope_count, input);
            true
        }
        "handler-bind" | "restart-bind" => {
            collect_handler_bind_references(
                view,
                symbol,
                output,
                shadowed_scope_count,
                input,
                head == "restart-bind",
            );
            true
        }
        "dolist" | "dotimes" => {
            collect_iteration_binding_references(view, symbol, output, shadowed_scope_count, input);
            true
        }
        "loop" => {
            collect_loop_references(view, symbol, output, shadowed_scope_count, input);
            true
        }
        "do" | "do*" => {
            collect_do_binding_references(
                view,
                symbol,
                output,
                shadowed_scope_count,
                input,
                head == "do*",
            );
            true
        }
        "prog" | "prog*" => {
            collect_prog_binding_references(
                view,
                symbol,
                output,
                shadowed_scope_count,
                input,
                head == "prog*",
            );
            true
        }
        "with-slots" | "with-accessors" => {
            collect_slot_binding_references(view, symbol, output, shadowed_scope_count, input);
            true
        }
        _ => false,
    }
}

fn collect_defmethod_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    let Some(parameter_index) = defmethod_specialized_lambda_list_index(view) else {
        return;
    };

    if specialized_parameter_form_binds(&view.children[parameter_index], symbol, input) {
        *shadowed_scope_count += 1;
        return;
    }

    for body in &view.children[parameter_index + 1..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
    }
}

fn defmethod_specialized_lambda_list_index(view: &ExpressionView) -> Option<usize> {
    view.children
        .iter()
        .enumerate()
        .skip(2)
        .find_map(|(index, child)| {
            (child.kind == ExpressionKind::List && child.delimiter == Some(Delimiter::Paren))
                .then_some(index)
        })
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
        if let Some(value) = &binding.value {
            collect_symbol_atom_spans_unshadowed(
                value,
                symbol,
                output,
                shadowed_scope_count,
                input,
            );
        }
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
        if let Some(value) = &binding.value {
            collect_symbol_atom_spans_unshadowed(
                value,
                symbol,
                output,
                shadowed_scope_count,
                input,
            );
        }
        if binding_binds(binding, symbol) {
            *shadowed_scope_count += 1;
            return;
        }
    }

    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
    }
}

fn collect_iteration_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };

    if let Some(source_form) = binding_form.children.get(1) {
        collect_symbol_atom_spans_unshadowed(
            source_form,
            symbol,
            output,
            shadowed_scope_count,
            input,
        );
    }

    if iteration_binding_form_binds(binding_form, symbol) {
        *shadowed_scope_count += 1;
        return;
    }

    if let Some(result_form) = binding_form.children.get(2) {
        collect_symbol_atom_spans_unshadowed(
            result_form,
            symbol,
            output,
            shadowed_scope_count,
            input,
        );
    }

    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
    }
}

fn collect_handler_bind_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
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
            collect_symbol_atom_spans_unshadowed(
                function_form,
                symbol,
                output,
                shadowed_scope_count,
                input,
            );
        }

        if include_restart_options {
            collect_restart_bind_option_value_references(
                spec,
                symbol,
                output,
                shadowed_scope_count,
                input,
            );
        }
    }

    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
    }
}

fn collect_restart_bind_option_value_references(
    spec: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    let mut index = 2;
    while index + 1 < spec.children.len() {
        collect_symbol_atom_spans_unshadowed(
            &spec.children[index + 1],
            symbol,
            output,
            shadowed_scope_count,
            input,
        );
        index += 2;
    }
}

fn collect_loop_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    let mut index = 1usize;

    while index < view.children.len() {
        let child = &view.children[index];

        if loop_keyword_is(child, "for") || loop_keyword_is(child, "as") {
            let binding_index = index + 1;
            let source_start = index + 2;
            let reference_start = loop_for_reference_start_index(&view.children, source_start);
            collect_loop_outer_references(
                &view.children[source_start..reference_start],
                symbol,
                output,
                shadowed_scope_count,
                input,
            );

            if view
                .children
                .get(binding_index)
                .is_some_and(|binding| binding_pattern_binds(binding, symbol, input))
            {
                *shadowed_scope_count += 1;
                return;
            }

            index = reference_start;
            continue;
        }

        if loop_keyword_is(child, "with") {
            let binding_index = index + 1;
            let init_start = index + 2;
            let reference_start = loop_with_reference_start_index(&view.children, init_start);
            collect_loop_outer_references(
                &view.children[init_start..reference_start],
                symbol,
                output,
                shadowed_scope_count,
                input,
            );

            if view
                .children
                .get(binding_index)
                .is_some_and(|binding| binding_pattern_binds(binding, symbol, input))
            {
                *shadowed_scope_count += 1;
                return;
            }

            index = reference_start;
            continue;
        }

        collect_symbol_atom_spans_unshadowed(child, symbol, output, shadowed_scope_count, input);
        index += 1;
    }
}

fn binding_pattern_binds(pattern: &ExpressionView, symbol: &SymbolName, input: &str) -> bool {
    binding_pattern_name_spans(pattern, input)
        .iter()
        .any(|name| name.name == symbol.as_str())
}

fn collect_loop_outer_references(
    forms: &[ExpressionView],
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    for form in forms {
        if loop_syntax_atom(form) {
            continue;
        }
        collect_symbol_atom_spans_unshadowed(form, symbol, output, shadowed_scope_count, input);
    }
}

fn loop_for_reference_start_index(children: &[ExpressionView], mut index: usize) -> usize {
    let Some(keyword) = children.get(index).and_then(atom_text) else {
        return index;
    };

    if matches_loop_keyword(keyword, &["in", "on", "across"]) {
        return (index + 2).min(children.len());
    }

    if matches_loop_keyword(keyword, &["=", "from", "downfrom", "upfrom"]) {
        index = (index + 2).min(children.len());
        while children.get(index).and_then(atom_text).is_some_and(|text| {
            matches_loop_keyword(text, &["to", "upto", "downto", "below", "above", "by"])
        }) {
            index = (index + 2).min(children.len());
        }
    }

    index
}

fn loop_with_reference_start_index(children: &[ExpressionView], index: usize) -> usize {
    if children
        .get(index)
        .is_some_and(|child| loop_keyword_is(child, "="))
    {
        return (index + 2).min(children.len());
    }

    index
}

fn loop_syntax_atom(view: &ExpressionView) -> bool {
    atom_text(view).is_some_and(|text| {
        matches_loop_keyword(
            text,
            &[
                "=", "in", "on", "across", "from", "downfrom", "upfrom", "to", "upto", "downto",
                "below", "above", "by",
            ],
        )
    })
}

fn loop_keyword_is(view: &ExpressionView, keyword: &str) -> bool {
    atom_text(view).is_some_and(|text| text.eq_ignore_ascii_case(keyword))
}

fn matches_loop_keyword(text: &str, keywords: &[&str]) -> bool {
    keywords
        .iter()
        .any(|keyword| text.eq_ignore_ascii_case(keyword))
}

fn collect_do_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
    sequential_scope: bool,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };

    if sequential_scope {
        for spec in &binding_form.children {
            if let Some(init_form) = variable_spec_init_form(spec) {
                collect_symbol_atom_spans_unshadowed(
                    init_form,
                    symbol,
                    output,
                    shadowed_scope_count,
                    input,
                );
            }
            if variable_spec_binds(spec, symbol) {
                *shadowed_scope_count += 1;
                return;
            }
        }
    } else {
        for spec in &binding_form.children {
            if let Some(init_form) = variable_spec_init_form(spec) {
                collect_symbol_atom_spans_unshadowed(
                    init_form,
                    symbol,
                    output,
                    shadowed_scope_count,
                    input,
                );
            }
        }
        if binding_form
            .children
            .iter()
            .any(|spec| variable_spec_binds(spec, symbol))
        {
            *shadowed_scope_count += 1;
            return;
        }
    }

    for spec in &binding_form.children {
        if let Some(step_form) = do_variable_spec_step_form(spec) {
            collect_symbol_atom_spans_unshadowed(
                step_form,
                symbol,
                output,
                shadowed_scope_count,
                input,
            );
        }
    }

    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
    }
}

fn collect_prog_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
    sequential_scope: bool,
) {
    let Some(binding_form) = view.children.get(1) else {
        return;
    };

    if sequential_scope {
        for spec in &binding_form.children {
            if let Some(init_form) = variable_spec_init_form(spec) {
                collect_symbol_atom_spans_unshadowed(
                    init_form,
                    symbol,
                    output,
                    shadowed_scope_count,
                    input,
                );
            }
            if variable_spec_binds(spec, symbol) {
                *shadowed_scope_count += 1;
                return;
            }
        }
    } else {
        for spec in &binding_form.children {
            if let Some(init_form) = variable_spec_init_form(spec) {
                collect_symbol_atom_spans_unshadowed(
                    init_form,
                    symbol,
                    output,
                    shadowed_scope_count,
                    input,
                );
            }
        }
        if binding_form
            .children
            .iter()
            .any(|spec| variable_spec_binds(spec, symbol))
        {
            *shadowed_scope_count += 1;
            return;
        }
    }

    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
    }
}

fn collect_slot_binding_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    let Some(slot_specs) = view.children.get(1) else {
        return;
    };
    let Some(instance_form) = view.children.get(2) else {
        return;
    };

    collect_symbol_atom_spans_unshadowed(
        instance_form,
        symbol,
        output,
        shadowed_scope_count,
        input,
    );

    if slot_specs
        .children
        .iter()
        .any(|spec| slot_spec_binds(spec, symbol))
    {
        *shadowed_scope_count += 1;
        return;
    }

    for body in &view.children[3..] {
        collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
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

fn collect_clause_form_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    if let Some(protected_form) = view.children.get(1) {
        collect_symbol_atom_spans_unshadowed(
            protected_form,
            symbol,
            output,
            shadowed_scope_count,
            input,
        );
    }

    for clause in &view.children[2..] {
        if clause.kind != ExpressionKind::List || clause.delimiter != Some(Delimiter::Paren) {
            collect_symbol_atom_spans_unshadowed(
                clause,
                symbol,
                output,
                shadowed_scope_count,
                input,
            );
            continue;
        }

        let Some(parameter_form) = clause.children.get(1) else {
            collect_symbol_atom_spans_unshadowed(
                clause,
                symbol,
                output,
                shadowed_scope_count,
                input,
            );
            continue;
        };

        if parameter_form_binds(parameter_form, symbol, input) {
            *shadowed_scope_count += 1;
            continue;
        }

        for body in &clause.children[2..] {
            collect_symbol_atom_spans_unshadowed(body, symbol, output, shadowed_scope_count, input);
        }
    }
}
