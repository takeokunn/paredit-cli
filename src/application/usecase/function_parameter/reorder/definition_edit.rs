use anyhow::{Context, Result};

use crate::domain::sexpr::{ExpressionKind, ExpressionView};

use super::super::list_edit::SpanEdit;
use super::parameter::ReorderableParameter;

pub(in crate::application::usecase::function_parameter) fn reorder_function_definition_edit(
    input: &str,
    container: &ExpressionView,
    parameters: &[ReorderableParameter],
    new_relative_order: &[usize],
    operation: &str,
) -> Result<SpanEdit> {
    if container.kind != ExpressionKind::List || container.delimiter.is_none() {
        anyhow::bail!("{operation} reorder target must be a list");
    }

    let start = container.span.start().get();
    let end = container.span.end().get();
    let open = &input[start..start + 1];
    let close = &input[end - 1..end];
    let reordered_parameter_items = new_relative_order
        .iter()
        .map(|&index| {
            let item_index = parameters[index].item_index;
            Ok(container.children[item_index].span.slice(input).to_owned())
        })
        .collect::<Result<Vec<_>>>()?;
    let mut reordered_parameter_items = reordered_parameter_items.into_iter();

    let items = container
        .children
        .iter()
        .enumerate()
        .map(|(item_index, child)| {
            if parameters
                .iter()
                .any(|parameter| parameter.item_index == item_index)
            {
                reordered_parameter_items.next().with_context(|| {
                    format!("{operation} definition reorder produced an incomplete parameter list")
                })
            } else {
                Ok(child.span.slice(input).to_owned())
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let replacement = if items.is_empty() {
        format!("{open}{close}")
    } else {
        format!("{open}{}{close}", items.join(" "))
    };
    Ok((container.span, replacement))
}
