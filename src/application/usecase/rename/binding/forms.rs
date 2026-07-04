use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::destructure::{binding_pattern_name_spans, lambda_list_name_spans};
use super::types::{BindingGroup, ParameterNameSpan};

pub(super) fn binding_groups(
    dialect: Dialect,
    binding_form: &ExpressionView,
    input: &str,
) -> Result<Vec<BindingGroup>> {
    match dialect {
        Dialect::Clojure | Dialect::Janet | Dialect::Fennel => {
            vector_let_binding_groups(binding_form, input)
        }
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Scheme | Dialect::Unknown => {
            list_pair_let_binding_groups(binding_form, input)
        }
    }
}

pub(super) fn generic_binding_groups(
    binding_form: &ExpressionView,
    input: &str,
) -> Result<Vec<BindingGroup>> {
    match binding_form.delimiter {
        Some(Delimiter::Bracket) => vector_let_binding_groups(binding_form, input),
        Some(Delimiter::Paren) => list_pair_let_binding_groups(binding_form, input),
        _ => anyhow::bail!("unknown binding form delimiter"),
    }
}

pub(super) fn parameter_name_spans(
    parameter_form: &ExpressionView,
    input: &str,
) -> Result<Vec<ParameterNameSpan>> {
    if parameter_form.kind != ExpressionKind::List {
        anyhow::bail!("parameter form must be a list");
    }

    Ok(lambda_list_name_spans(parameter_form, input))
}

pub(super) fn parameter_form_binds(
    parameter_form: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
) -> bool {
    parameter_form.kind == ExpressionKind::List
        && lambda_list_name_spans(parameter_form, input)
            .iter()
            .any(|name| name.name == symbol.as_str())
}

pub(super) fn binding_binds(binding: &BindingGroup, symbol: &SymbolName) -> bool {
    binding
        .names
        .iter()
        .any(|name| name.name == symbol.as_str())
}

fn vector_let_binding_groups(
    binding_form: &ExpressionView,
    input: &str,
) -> Result<Vec<BindingGroup>> {
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
            let names = binding_pattern_name_spans(&pair[0], input);
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

fn list_pair_let_binding_groups(
    binding_form: &ExpressionView,
    input: &str,
) -> Result<Vec<BindingGroup>> {
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
            let names = binding_pattern_name_spans(&pair.children[0], input);
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
