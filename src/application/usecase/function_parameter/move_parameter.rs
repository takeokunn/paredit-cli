use anyhow::{Context, Result};

use crate::domain::sexpr::SyntaxTree;

use super::calls::{move_function_parameter_call_edit, resolve_function_call_paths};
use super::definition::{
    find_unique_parameter_item_index, parse_move_function_parameter_definition,
};
use super::list_edit::{
    apply_byte_span_edits, ensure_non_overlapping_spans, move_list_item_edit, spans_overlap,
};
use super::types::{MoveFunctionParameterPlan, MoveFunctionParameterRequest};

pub fn plan_move_function_parameter(
    request: MoveFunctionParameterRequest<'_>,
) -> Result<MoveFunctionParameterPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    let definition_selection = tree.select_path(&request.definition_path)?;
    let target =
        parse_move_function_parameter_definition(request.dialect, definition_selection.view())?;
    let parameter_item_index = find_unique_parameter_item_index(
        &target.parameter_container,
        target.protected_prefix_count,
        &request.name,
        "move-function-parameter",
    )?;
    let from_index = parameter_item_index - target.protected_prefix_count;
    let parameter_count = target
        .parameter_container
        .children
        .len()
        .saturating_sub(target.protected_prefix_count);
    if request.to_index >= parameter_count {
        anyhow::bail!(
            "move-function-parameter target index {} is out of bounds for {} required parameters",
            request.to_index,
            parameter_count
        );
    }
    let call_paths = resolve_function_call_paths(
        &tree,
        request.call_paths,
        request.all_calls,
        target.definition_span,
        &target.function_name,
        "move-function-parameter",
    )?;

    let mut edits = Vec::with_capacity(call_paths.len() + 1);
    if from_index != request.to_index {
        edits.push(move_list_item_edit(
            request.input,
            &target.parameter_container,
            parameter_item_index,
            target.protected_prefix_count,
            target.protected_prefix_count + request.to_index,
            "move-function-parameter",
        )?);
    }

    let mut call_spans = Vec::with_capacity(call_paths.len());
    let mut moved_arguments = Vec::with_capacity(call_paths.len());
    for call_path in &call_paths {
        let call_selection = tree.select_path(call_path)?;
        if spans_overlap(target.definition_span, call_selection.span()) {
            anyhow::bail!(
                "move-function-parameter call path {} overlaps the selected definition",
                call_path
            );
        }
        let (call_span, moved_argument, edit) = move_function_parameter_call_edit(
            request.input,
            call_selection.view(),
            &target.function_name,
            from_index,
            request.to_index,
        )?;
        call_spans.push(call_span);
        moved_arguments.push(moved_argument);
        if let Some(edit) = edit {
            edits.push(edit);
        }
    }

    let mut sorted_call_spans = call_spans.clone();
    sorted_call_spans.sort_by_key(|span| span.start());
    ensure_non_overlapping_spans(sorted_call_spans)?;
    let rewritten = apply_byte_span_edits(request.input, edits)?;
    SyntaxTree::parse(&rewritten)
        .context("move-function-parameter output is not a valid S-expression document")?;

    let changed = rewritten != request.input;
    Ok(MoveFunctionParameterPlan {
        dialect: request.dialect,
        definition_path: request.definition_path,
        call_paths,
        all_calls: request.all_calls,
        definition_span: target.definition_span,
        parameter_list_span: target.parameter_container.span,
        call_spans,
        function_name: target.function_name,
        parameter_name: request.name,
        from_index,
        to_index: request.to_index,
        moved_arguments,
        rewritten,
        changed,
    })
}
