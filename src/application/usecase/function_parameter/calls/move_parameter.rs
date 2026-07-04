use anyhow::Result;

use crate::application::usecase::function_parameter::list_edit::{SpanEdit, move_list_item_edit};
use crate::domain::sexpr::{ByteSpan, ExpressionView, SymbolName};

use super::validation::ensure_matching_function_call;

pub(in crate::application::usecase::function_parameter) type MoveArgumentEdit =
    (ByteSpan, String, Option<SpanEdit>);

pub(in crate::application::usecase::function_parameter) fn move_function_parameter_call_edit(
    input: &str,
    view: ExpressionView,
    function_name: &SymbolName,
    from_index: usize,
    to_index: usize,
) -> Result<MoveArgumentEdit> {
    ensure_matching_function_call(&view, function_name, "move-function-parameter")?;

    let argument_count = view.children.len().saturating_sub(1);
    if from_index >= argument_count {
        anyhow::bail!(
            "move-function-parameter call to '{}' at {}..{} does not have argument at parameter index {}",
            function_name,
            view.span.start().get(),
            view.span.end().get(),
            from_index
        );
    }
    if to_index >= argument_count {
        anyhow::bail!(
            "move-function-parameter target index {} is out of bounds for call to '{}' with {} arguments",
            to_index,
            function_name,
            argument_count
        );
    }

    let argument_item_index = from_index + 1;
    let argument = &view.children[argument_item_index];
    let moved_argument = argument.span.slice(input).to_owned();
    if from_index == to_index {
        return Ok((view.span, moved_argument, None));
    }
    let edit = move_list_item_edit(
        input,
        &view,
        argument_item_index,
        1,
        to_index + 1,
        "move-function-parameter",
    )?;
    Ok((view.span, moved_argument, Some(edit)))
}
