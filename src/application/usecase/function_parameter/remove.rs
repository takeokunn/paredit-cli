use anyhow::{Context, Result};

use crate::domain::sexpr::SyntaxTree;

use super::calls::{remove_function_parameter_call_edit, resolve_function_call_paths};
use super::definition::{
    find_unique_parameter_item_index, parse_remove_function_parameter_definition,
};
use super::list_edit::{
    apply_byte_span_edits, ensure_non_overlapping_spans, removal_edit_for_list_item, spans_overlap,
};
use super::types::{RemoveFunctionParameterPlan, RemoveFunctionParameterRequest};

pub fn plan_remove_function_parameter(
    request: RemoveFunctionParameterRequest<'_>,
) -> Result<RemoveFunctionParameterPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    let definition_selection = tree.select_path(&request.definition_path)?;
    let target =
        parse_remove_function_parameter_definition(request.dialect, definition_selection.view())?;
    let parameter_item_index = find_unique_parameter_item_index(
        &target.parameter_container,
        target.protected_prefix_count,
        &request.name,
        "remove-function-parameter",
    )?;
    let parameter_index = parameter_item_index - target.protected_prefix_count;
    let call_paths = resolve_function_call_paths(
        &tree,
        request.call_paths,
        request.all_calls,
        target.definition_span,
        &target.function_name,
        "remove-function-parameter",
    )?;

    let mut edits = Vec::with_capacity(call_paths.len() + 1);
    edits.push(removal_edit_for_list_item(
        &target.parameter_container,
        parameter_item_index,
    )?);

    let mut call_spans = Vec::with_capacity(call_paths.len());
    let mut removed_arguments = Vec::with_capacity(call_paths.len());
    for call_path in &call_paths {
        let call_selection = tree.select_path(call_path)?;
        if spans_overlap(target.definition_span, call_selection.span()) {
            anyhow::bail!(
                "remove-function-parameter call path {} overlaps the selected definition",
                call_path
            );
        }
        let (call_span, removed_argument, edit) = remove_function_parameter_call_edit(
            request.input,
            call_selection.view(),
            &target.function_name,
            parameter_index,
            request.allow_missing_argument,
        )?;
        call_spans.push(call_span);
        removed_arguments.push(removed_argument);
        if let Some(edit) = edit {
            edits.push(edit);
        }
    }

    let mut sorted_call_spans = call_spans.clone();
    sorted_call_spans.sort_by_key(|span| span.start());
    ensure_non_overlapping_spans(sorted_call_spans)?;
    let rewritten = apply_byte_span_edits(request.input, edits)?;
    SyntaxTree::parse(&rewritten)
        .context("remove-function-parameter output is not a valid S-expression document")?;

    let changed = rewritten != request.input;
    Ok(RemoveFunctionParameterPlan {
        dialect: request.dialect,
        definition_path: request.definition_path,
        call_paths,
        all_calls: request.all_calls,
        definition_span: target.definition_span,
        parameter_list_span: target.parameter_container.span,
        call_spans,
        function_name: target.function_name,
        parameter_name: request.name,
        parameter_index,
        removed_arguments,
        rewritten,
        changed,
    })
}
