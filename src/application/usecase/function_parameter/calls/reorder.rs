use anyhow::Result;

use crate::application::usecase::function_parameter::list_edit::{
    SpanEdit, reorder_list_items_edit,
};
use crate::domain::sexpr::{ByteSpan, ExpressionView, SymbolName};

use super::validation::ensure_matching_function_call;

pub(in crate::application::usecase::function_parameter) type ReorderArgumentEdit =
    (ByteSpan, Vec<String>, Option<SpanEdit>);

pub(in crate::application::usecase::function_parameter) fn reorder_function_parameter_call_edit(
    input: &str,
    view: ExpressionView,
    function_name: &SymbolName,
    new_relative_order: &[usize],
) -> Result<ReorderArgumentEdit> {
    ensure_matching_function_call(&view, function_name, "reorder-function-parameters")?;

    let argument_count = view.children.len().saturating_sub(1);
    if argument_count != new_relative_order.len() {
        anyhow::bail!(
            "reorder-function-parameters call to '{}' at {}..{} has {} arguments but needs exactly {}",
            function_name,
            view.span.start().get(),
            view.span.end().get(),
            argument_count,
            new_relative_order.len()
        );
    }

    let old_arguments = view.children[1..]
        .iter()
        .map(|child| child.span.slice(input).to_owned())
        .collect::<Vec<_>>();
    let reordered_arguments = new_relative_order
        .iter()
        .map(|&index| old_arguments[index].clone())
        .collect::<Vec<_>>();
    if new_relative_order
        .iter()
        .copied()
        .eq(0..new_relative_order.len())
    {
        return Ok((view.span, reordered_arguments, None));
    }

    let edit = reorder_list_items_edit(
        input,
        &view,
        1,
        new_relative_order,
        "reorder-function-parameters",
    )?;
    Ok((view.span, reordered_arguments, Some(edit)))
}
