use anyhow::Result;

use crate::application::usecase::function_parameter::list_edit::{
    SpanEdit, removal_edit_for_list_item,
};
use crate::domain::sexpr::{ByteSpan, ExpressionView, SymbolName};

use super::validation::ensure_matching_function_call;

pub(in crate::application::usecase::function_parameter) type RemoveArgumentEdit =
    (ByteSpan, Option<String>, Option<SpanEdit>);

pub(in crate::application::usecase::function_parameter) fn remove_function_parameter_call_edit(
    input: &str,
    view: ExpressionView,
    function_name: &SymbolName,
    parameter_index: usize,
    allow_missing_argument: bool,
) -> Result<RemoveArgumentEdit> {
    ensure_matching_function_call(&view, function_name, "remove-function-parameter")?;

    let argument_item_index = parameter_index + 1;
    let Some(argument) = view.children.get(argument_item_index) else {
        if allow_missing_argument {
            return Ok((view.span, None, None));
        }
        anyhow::bail!(
            "remove-function-parameter call to '{}' at {}..{} does not have argument at parameter index {}",
            function_name,
            view.span.start().get(),
            view.span.end().get(),
            parameter_index
        );
    };
    let removed_argument = argument.span.slice(input).to_owned();
    let edit = removal_edit_for_list_item(&view, argument_item_index)?;
    Ok((view.span, Some(removed_argument), Some(edit)))
}
