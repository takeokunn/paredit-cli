use anyhow::Result;

use crate::application::usecase::function_parameter::list_edit::{SpanEdit, swap_list_item_edit};
use crate::domain::sexpr::{ByteSpan, ExpressionView, SymbolName};

use super::validation::ensure_matching_function_call;

pub(in crate::application::usecase::function_parameter) type SwapArgumentEdit =
    (ByteSpan, (String, String), Option<SpanEdit>);

pub(in crate::application::usecase::function_parameter) fn swap_function_parameter_call_edit(
    input: &str,
    view: ExpressionView,
    function_name: &SymbolName,
    left_index: usize,
    right_index: usize,
) -> Result<SwapArgumentEdit> {
    ensure_matching_function_call(&view, function_name, "swap-function-parameters")?;

    let argument_count = view.children.len().saturating_sub(1);
    let required_count = left_index.max(right_index) + 1;
    if argument_count < required_count {
        anyhow::bail!(
            "swap-function-parameters call to '{}' at {}..{} has {} arguments but needs parameter index {}",
            function_name,
            view.span.start().get(),
            view.span.end().get(),
            argument_count,
            required_count - 1
        );
    }

    let left_item_index = left_index + 1;
    let right_item_index = right_index + 1;
    let left_argument = view.children[left_item_index].span.slice(input).to_owned();
    let right_argument = view.children[right_item_index].span.slice(input).to_owned();
    if left_index == right_index {
        return Ok((view.span, (left_argument, right_argument), None));
    }

    let edit = swap_list_item_edit(
        input,
        &view,
        left_item_index,
        right_item_index,
        1,
        "swap-function-parameters",
    )?;
    Ok((view.span, (left_argument, right_argument), Some(edit)))
}
