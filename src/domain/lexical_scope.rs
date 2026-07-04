//! Lexical binding and scope helpers shared by refactoring use cases.

use anyhow::Result;

use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

#[derive(Debug, Clone)]
struct BindingGroup {
    names: Vec<String>,
    value: ExpressionView,
}

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

fn generic_binding_groups(binding_form: &ExpressionView) -> Result<Vec<BindingGroup>> {
    match binding_form.delimiter {
        Some(Delimiter::Bracket) => vector_let_binding_groups(binding_form),
        Some(Delimiter::Paren) => list_pair_let_binding_groups(binding_form),
        _ => anyhow::bail!("unknown binding form delimiter"),
    }
}

fn vector_let_binding_groups(binding_form: &ExpressionView) -> Result<Vec<BindingGroup>> {
    if binding_form.kind != ExpressionKind::List
        || binding_form.delimiter != Some(Delimiter::Bracket)
    {
        anyhow::bail!("dialect expects vector let bindings: [name value ...]");
    }
    if binding_form.children.len() % 2 != 0 {
        anyhow::bail!("vector let binding form must contain name/value pairs");
    }

    binding_form
        .children
        .chunks_exact(2)
        .map(|pair| {
            let names = binding_pattern_names(&pair[0]);
            if names.is_empty() {
                anyhow::bail!("let binding pattern must contain at least one binding name");
            }
            Ok(BindingGroup {
                names,
                value: pair[1].clone(),
            })
        })
        .collect()
}

fn list_pair_let_binding_groups(binding_form: &ExpressionView) -> Result<Vec<BindingGroup>> {
    if binding_form.kind != ExpressionKind::List || binding_form.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!("dialect expects list-pair let bindings: ((name value) ...)");
    }

    binding_form
        .children
        .iter()
        .map(|pair| {
            if pair.kind != ExpressionKind::List || pair.delimiter != Some(Delimiter::Paren) {
                anyhow::bail!("let binding must be a (name value) pair");
            }
            if pair.children.len() != 2 {
                anyhow::bail!("let binding pair must contain a name and value");
            }
            let names = binding_pattern_names(&pair.children[0]);
            if names.is_empty() {
                anyhow::bail!("let binding pattern must contain at least one binding name");
            }
            Ok(BindingGroup {
                names,
                value: pair.children[1].clone(),
            })
        })
        .collect()
}

fn parameter_form_binds(parameter_form: &ExpressionView, symbol: &SymbolName) -> bool {
    parameter_form.kind == ExpressionKind::List
        && parameter_form
            .children
            .iter()
            .flat_map(binding_pattern_names)
            .any(|name| name == symbol.as_str())
}

fn binding_binds(binding: &BindingGroup, symbol: &SymbolName) -> bool {
    binding.names.iter().any(|name| name == symbol.as_str())
}

fn binding_pattern_names(pattern: &ExpressionView) -> Vec<String> {
    let mut names = Vec::new();
    collect_binding_pattern_names(pattern, &mut names);
    names
}

fn collect_binding_pattern_names(pattern: &ExpressionView, output: &mut Vec<String>) {
    if let Some(name) = atom_text(pattern) {
        if !is_binding_pattern_marker(name) {
            output.push(name.to_owned());
        }
        return;
    }

    let mut index = 0usize;
    while index < pattern.children.len() {
        let child = &pattern.children[index];
        if let Some(marker) = atom_text(child) {
            if marker == ":keys" {
                if let Some(keys_form) = pattern.children.get(index + 1) {
                    collect_clojure_keys_shorthand_names(keys_form, output);
                }
                index += 2;
                continue;
            }
            if matches!(marker, ":strs" | ":syms") {
                index += 2;
                continue;
            }
            if marker == ":as" {
                if let Some(next) = pattern.children.get(index + 1) {
                    collect_binding_pattern_names(next, output);
                    index += 2;
                    continue;
                }
            }
        }
        collect_binding_pattern_names(child, output);
        index += 1;
    }
}

fn collect_clojure_keys_shorthand_names(keys_form: &ExpressionView, output: &mut Vec<String>) {
    if keys_form.kind != ExpressionKind::List || keys_form.delimiter != Some(Delimiter::Bracket) {
        return;
    }

    for key in &keys_form.children {
        let Some(name) = atom_text(key) else {
            continue;
        };
        if !is_binding_pattern_marker(name) {
            output.push(name.to_owned());
        }
    }
}

fn is_binding_pattern_marker(name: &str) -> bool {
    name == "_" || name.starts_with('&') || name.starts_with(':')
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::domain::sexpr::{Path, SyntaxTree};

    fn selected_form(input: &str) -> ExpressionView {
        let tree = SyntaxTree::parse(input).expect("parse");
        tree.select_path(&"0".parse::<Path>().expect("path"))
            .expect("select")
            .view()
    }

    fn reference_texts(input: &str, symbol: &str) -> Vec<String> {
        let view = selected_form(input);
        let symbol = SymbolName::new(symbol).expect("symbol");
        let mut spans = Vec::new();
        collect_unshadowed_symbol_references(&view, &symbol, input, &mut spans);
        spans
            .into_iter()
            .map(|span| span.slice(input).to_owned())
            .collect()
    }

    #[test]
    fn skips_shadowed_lambda_parameter_references() {
        let input = "(list x (lambda (x) x))";

        assert_eq!(reference_texts(input, "x"), vec!["x"]);
    }

    #[test]
    fn sequential_let_stops_after_shadowing_binding() {
        let input = "(let* ((y x) (x 2)) (list x y))";

        assert_eq!(reference_texts(input, "x"), vec!["x"]);
    }

    #[test]
    fn clojure_destructuring_shadows_keys_shorthand() {
        let input = "(list x (fn [{:keys [x] :as m}] x m))";

        assert_eq!(reference_texts(input, "x"), vec!["x"]);
    }

    proptest! {
        #[test]
        fn pbt_shadowed_lambda_references_are_not_counted(count in 1usize..12) {
            let lambdas = std::iter::repeat("(lambda (x) x)")
                .take(count)
                .collect::<Vec<_>>()
                .join(" ");
            let input = format!("(list x {lambdas})");

            prop_assert_eq!(reference_texts(&input, "x"), vec!["x"]);
        }
    }
}
