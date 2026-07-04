use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};

use crate::domain::sexpr::{ExpressionView, SymbolName, SyntaxTree};

use super::calls::{reorder_function_parameter_call_edit, resolve_function_call_paths};
use super::definition::parse_reorder_function_parameters_definition;
use super::list_edit::{
    SpanEdit, apply_byte_span_edits, atom_text, ensure_non_overlapping_spans,
    reorder_list_items_edit, spans_overlap,
};
use super::types::{ReorderFunctionParametersPlan, ReorderFunctionParametersRequest};

pub fn plan_reorder_function_parameters(
    request: ReorderFunctionParametersRequest<'_>,
) -> Result<ReorderFunctionParametersPlan> {
    if request.parameter_order.is_empty() {
        anyhow::bail!("reorder-function-parameters requires at least one --parameter");
    }

    let tree = SyntaxTree::parse(request.input)?;
    let definition_selection = tree.select_path(&request.definition_path)?;
    let target =
        parse_reorder_function_parameters_definition(request.dialect, definition_selection.view())?;
    let old_parameter_order = required_parameter_names(
        &target.parameter_container,
        target.protected_prefix_count,
        "reorder-function-parameters",
    )?;
    let new_relative_order =
        build_new_relative_order(&old_parameter_order, &request.parameter_order)?;
    let call_paths = resolve_function_call_paths(
        &tree,
        request.dialect,
        request.call_paths,
        request.all_calls,
        target.definition_span,
        &target.function_name,
        "reorder-function-parameters",
    )?;

    let mut edits = Vec::<SpanEdit>::with_capacity(call_paths.len() + 1);
    if !is_identity_order(&new_relative_order) {
        edits.push(reorder_list_items_edit(
            request.input,
            &target.parameter_container,
            target.protected_prefix_count,
            &new_relative_order,
            "reorder-function-parameters",
        )?);
    }

    let mut call_spans = Vec::with_capacity(call_paths.len());
    let mut reordered_arguments = Vec::with_capacity(call_paths.len());
    for call_path in &call_paths {
        let call_selection = tree.select_path(call_path)?;
        if spans_overlap(target.definition_span, call_selection.span()) {
            anyhow::bail!(
                "reorder-function-parameters call path {} overlaps the selected definition",
                call_path
            );
        }
        let (call_span, reordered_argument, edit) = reorder_function_parameter_call_edit(
            request.input,
            call_selection.view(),
            &target.function_name,
            &new_relative_order,
        )?;
        call_spans.push(call_span);
        reordered_arguments.push(reordered_argument);
        if let Some(edit) = edit {
            edits.push(edit);
        }
    }

    let mut sorted_call_spans = call_spans.clone();
    sorted_call_spans.sort_by_key(|span| span.start());
    ensure_non_overlapping_spans(sorted_call_spans)?;
    let rewritten = apply_byte_span_edits(request.input, edits)?;
    SyntaxTree::parse(&rewritten)
        .context("reorder-function-parameters output is not a valid S-expression document")?;

    let changed = rewritten != request.input;
    Ok(ReorderFunctionParametersPlan {
        dialect: request.dialect,
        definition_path: request.definition_path,
        call_paths,
        all_calls: request.all_calls,
        definition_span: target.definition_span,
        parameter_list_span: target.parameter_container.span,
        call_spans,
        function_name: target.function_name,
        old_parameter_order,
        new_parameter_order: request.parameter_order,
        reordered_arguments,
        rewritten,
        changed,
    })
}

fn required_parameter_names(
    container: &ExpressionView,
    protected_prefix_count: usize,
    operation: &str,
) -> Result<Vec<SymbolName>> {
    container.children[protected_prefix_count..]
        .iter()
        .map(|child| {
            let name = atom_text(child).with_context(|| {
                format!("{operation} currently supports only simple symbol parameters")
            })?;
            if name.starts_with('&') {
                anyhow::bail!(
                    "{operation} currently supports only required parameters; found marker {}",
                    name
                );
            }
            SymbolName::new(name.to_owned())
                .with_context(|| format!("{operation} found invalid parameter symbol '{}'", name))
        })
        .collect()
}

fn build_new_relative_order(
    old_order: &[SymbolName],
    new_order: &[SymbolName],
) -> Result<Vec<usize>> {
    if new_order.len() != old_order.len() {
        anyhow::bail!(
            "reorder-function-parameters requested {} parameters but definition has {}",
            new_order.len(),
            old_order.len()
        );
    }

    let mut old_indexes = BTreeMap::new();
    for (index, name) in old_order.iter().enumerate() {
        if old_indexes.insert(name.as_str(), index).is_some() {
            anyhow::bail!(
                "reorder-function-parameters cannot reorder duplicate definition parameter '{}'",
                name
            );
        }
    }

    let mut requested_names = BTreeSet::new();
    let mut relative_order = Vec::with_capacity(new_order.len());
    for name in new_order {
        if !requested_names.insert(name.as_str()) {
            anyhow::bail!(
                "reorder-function-parameters requested parameter '{}' more than once",
                name
            );
        }
        let index = old_indexes.get(name.as_str()).copied().with_context(|| {
            format!(
                "reorder-function-parameters requested unknown parameter '{}'",
                name
            )
        })?;
        relative_order.push(index);
    }

    for name in old_order {
        if !requested_names.contains(name.as_str()) {
            anyhow::bail!(
                "reorder-function-parameters missing parameter '{}' from requested order",
                name
            );
        }
    }

    Ok(relative_order)
}

fn is_identity_order(order: &[usize]) -> bool {
    order.iter().copied().eq(0..order.len())
}
