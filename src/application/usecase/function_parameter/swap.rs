use anyhow::{Context, Result};

use crate::domain::sexpr::SyntaxTree;

use super::calls::{resolve_function_call_paths, swap_function_parameter_call_edit};
use super::definition::{
    find_unique_parameter_item_index, parse_swap_function_parameters_definition,
};
use super::list_edit::{
    apply_byte_span_edits, ensure_non_overlapping_spans, spans_overlap, swap_list_item_edit,
};
use super::types::{SwapFunctionParametersPlan, SwapFunctionParametersRequest};

pub fn plan_swap_function_parameters(
    request: SwapFunctionParametersRequest<'_>,
) -> Result<SwapFunctionParametersPlan> {
    if request.left_name == request.right_name {
        anyhow::bail!("swap-function-parameters requires two distinct parameter names");
    }

    let tree = SyntaxTree::parse(request.input)?;
    let definition_selection = tree.select_path(&request.definition_path)?;
    let target =
        parse_swap_function_parameters_definition(request.dialect, definition_selection.view())?;
    let left_item_index = find_unique_parameter_item_index(
        &target.parameter_container,
        target.protected_prefix_count,
        &request.left_name,
        "swap-function-parameters",
    )?;
    let right_item_index = find_unique_parameter_item_index(
        &target.parameter_container,
        target.protected_prefix_count,
        &request.right_name,
        "swap-function-parameters",
    )?;
    let left_index = left_item_index - target.protected_prefix_count;
    let right_index = right_item_index - target.protected_prefix_count;
    let call_paths = resolve_function_call_paths(
        &tree,
        request.call_paths,
        request.all_calls,
        target.definition_span,
        &target.function_name,
        "swap-function-parameters",
    )?;

    let mut edits = Vec::with_capacity(call_paths.len() + 1);
    edits.push(swap_list_item_edit(
        request.input,
        &target.parameter_container,
        left_item_index,
        right_item_index,
        target.protected_prefix_count,
        "swap-function-parameters",
    )?);

    let mut call_spans = Vec::with_capacity(call_paths.len());
    let mut swapped_arguments = Vec::with_capacity(call_paths.len());
    for call_path in &call_paths {
        let call_selection = tree.select_path(call_path)?;
        if spans_overlap(target.definition_span, call_selection.span()) {
            anyhow::bail!(
                "swap-function-parameters call path {} overlaps the selected definition",
                call_path
            );
        }
        let (call_span, swapped_argument, edit) = swap_function_parameter_call_edit(
            request.input,
            call_selection.view(),
            &target.function_name,
            left_index,
            right_index,
        )?;
        call_spans.push(call_span);
        swapped_arguments.push(swapped_argument);
        if let Some(edit) = edit {
            edits.push(edit);
        }
    }

    let mut sorted_call_spans = call_spans.clone();
    sorted_call_spans.sort_by_key(|span| span.start());
    ensure_non_overlapping_spans(sorted_call_spans)?;
    let rewritten = apply_byte_span_edits(request.input, edits)?;
    SyntaxTree::parse(&rewritten)
        .context("swap-function-parameters output is not a valid S-expression document")?;

    let changed = rewritten != request.input;
    Ok(SwapFunctionParametersPlan {
        dialect: request.dialect,
        definition_path: request.definition_path,
        call_paths,
        all_calls: request.all_calls,
        definition_span: target.definition_span,
        parameter_list_span: target.parameter_container.span,
        call_spans,
        function_name: target.function_name,
        left_name: request.left_name,
        right_name: request.right_name,
        left_index,
        right_index,
        swapped_arguments,
        rewritten,
        changed,
    })
}
