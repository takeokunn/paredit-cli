mod destructure;
mod forms;
mod rewrite;
mod scope;
mod types;

use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionView, SymbolName};

use super::selection::list_head;
use forms::{binding_groups, parameter_name_spans};
use scope::collect_symbol_atom_spans_unshadowed;
pub(super) use types::BindingRenameParts;

pub(super) fn binding_rename_parts(
    dialect: Dialect,
    view: &ExpressionView,
    from: &SymbolName,
    input: &str,
) -> Result<BindingRenameParts> {
    let form = list_head(view)
        .context("selected form is not a supported binding form")?
        .to_owned();

    match form.as_str() {
        "let" | "let*" => let_binding_rename_parts(dialect, view, from, form, input),
        "lambda" | "fn" => parameter_binding_rename_parts(view, from, form, 1, 2, input),
        "defun" | "defmacro" => parameter_binding_rename_parts(view, from, form, 2, 3, input),
        _ => anyhow::bail!("selected form is not a supported binding form"),
    }
}

fn let_binding_rename_parts(
    dialect: Dialect,
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let binding_form = view
        .children
        .get(1)
        .context("selected let form must contain bindings")?;
    let bindings = binding_groups(dialect, binding_form, input)?;
    let (target_index, target) = bindings
        .iter()
        .enumerate()
        .find_map(|(index, binding)| {
            binding
                .names
                .iter()
                .find(|name| name.name == from.as_str())
                .map(|name| (index, name))
        })
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected let"))?;

    let sequential_scope = form == "let*" || binding_form.delimiter == Some(Delimiter::Bracket);
    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    if sequential_scope {
        for later in bindings.iter().skip(target_index + 1) {
            collect_symbol_atom_spans_unshadowed(
                &later.value,
                from,
                &mut reference_spans,
                &mut shadowed_scope_count,
                input,
            );
        }
    }

    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(
            body,
            from,
            &mut reference_spans,
            &mut shadowed_scope_count,
            input,
        );
    }
    reference_spans.sort_by_key(|span| span.start());

    Ok(BindingRenameParts {
        form,
        form_span: view.span,
        binding_span: target.name_span,
        binding_edit: target.binding_edit.clone(),
        reference_spans,
        shadowed_scope_count,
    })
}

fn parameter_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    parameter_index: usize,
    body_start_index: usize,
    input: &str,
) -> Result<BindingRenameParts> {
    let parameter_form = view
        .children
        .get(parameter_index)
        .with_context(|| format!("selected {form} form must contain parameters"))?;
    let parameters = parameter_name_spans(parameter_form, input)?;
    let target = parameters
        .iter()
        .find(|parameter| parameter.name == from.as_str())
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    for body in &view.children[body_start_index..] {
        collect_symbol_atom_spans_unshadowed(
            body,
            from,
            &mut reference_spans,
            &mut shadowed_scope_count,
            input,
        );
    }
    reference_spans.sort_by_key(|span| span.start());

    Ok(BindingRenameParts {
        form,
        form_span: view.span,
        binding_span: target.name_span,
        binding_edit: target.binding_edit.clone(),
        reference_spans,
        shadowed_scope_count,
    })
}
