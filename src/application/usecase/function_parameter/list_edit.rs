use anyhow::Result;

use crate::domain::sexpr::{ByteOffset, ByteSpan, ExpressionKind, ExpressionView};

use super::FunctionParameterInsert;

pub(super) type SpanEdit = (ByteSpan, String);

pub(super) fn insertion_edit_for_list_item(
    container: &ExpressionView,
    protected_prefix_count: usize,
    value: &str,
    insert: FunctionParameterInsert,
) -> Result<SpanEdit> {
    if container.kind != ExpressionKind::List || container.delimiter.is_none() {
        anyhow::bail!("add-function-parameter insertion target must be a list");
    }
    if container.span.len() < 2 {
        anyhow::bail!("add-function-parameter insertion target has an invalid span");
    }

    let item_start = protected_prefix_count.min(container.children.len());
    let items = &container.children[item_start..];
    let (offset, replacement) = match insert {
        FunctionParameterInsert::Start => match items.first() {
            Some(first) => (first.span.start().get(), format!("{value} ")),
            None => {
                let close = container.span.end().get() - 1;
                let prefix = if container.children.is_empty() {
                    ""
                } else {
                    " "
                };
                (close, format!("{prefix}{value}"))
            }
        },
        FunctionParameterInsert::End => {
            let close = container.span.end().get() - 1;
            let prefix = if container.children.is_empty() {
                ""
            } else {
                " "
            };
            (close, format!("{prefix}{value}"))
        }
    };

    Ok((
        ByteSpan::new(ByteOffset::new(offset), ByteOffset::new(offset)),
        replacement,
    ))
}

pub(super) fn removal_edit_for_list_item(
    container: &ExpressionView,
    item_index: usize,
) -> Result<SpanEdit> {
    if container.kind != ExpressionKind::List || container.delimiter.is_none() {
        anyhow::bail!("remove-function-parameter removal target must be a list");
    }
    if item_index >= container.children.len() {
        anyhow::bail!(
            "remove-function-parameter removal item index {} is out of bounds",
            item_index
        );
    }

    let item = &container.children[item_index];
    let span = if container.children.len() == 1 {
        item.span
    } else if item_index == 0 {
        let next = &container.children[1];
        ByteSpan::new(item.span.start(), next.span.start())
    } else {
        let previous = &container.children[item_index - 1];
        ByteSpan::new(previous.span.end(), item.span.end())
    };

    Ok((span, String::new()))
}

pub(super) fn move_list_item_edit(
    input: &str,
    container: &ExpressionView,
    from_item_index: usize,
    protected_prefix_count: usize,
    to_item_index: usize,
    operation: &str,
) -> Result<SpanEdit> {
    if container.kind != ExpressionKind::List || container.delimiter.is_none() {
        anyhow::bail!("{operation} move target must be a list");
    }
    if container.span.len() < 2 {
        anyhow::bail!("{operation} move target has an invalid span");
    }
    if from_item_index >= container.children.len() {
        anyhow::bail!(
            "{operation} source item index {} is out of bounds",
            from_item_index
        );
    }
    if to_item_index >= container.children.len() {
        anyhow::bail!(
            "{operation} target item index {} is out of bounds",
            to_item_index
        );
    }
    if from_item_index < protected_prefix_count || to_item_index < protected_prefix_count {
        anyhow::bail!("{operation} cannot move protected list prefix items");
    }

    let start = container.span.start().get();
    let end = container.span.end().get();
    let open = &input[start..start + 1];
    let close = &input[end - 1..end];
    let mut protected_items = container.children[..protected_prefix_count]
        .iter()
        .map(|child| child.span.slice(input).to_owned())
        .collect::<Vec<_>>();
    let mut movable_items = container.children[protected_prefix_count..]
        .iter()
        .map(|child| child.span.slice(input).to_owned())
        .collect::<Vec<_>>();
    let moved = movable_items.remove(from_item_index - protected_prefix_count);
    movable_items.insert(to_item_index - protected_prefix_count, moved);
    protected_items.extend(movable_items);

    let replacement = if protected_items.is_empty() {
        format!("{open}{close}")
    } else {
        format!("{open}{}{close}", protected_items.join(" "))
    };
    Ok((container.span, replacement))
}

pub(super) fn swap_list_item_edit(
    input: &str,
    container: &ExpressionView,
    left_item_index: usize,
    right_item_index: usize,
    protected_prefix_count: usize,
    operation: &str,
) -> Result<SpanEdit> {
    if container.kind != ExpressionKind::List || container.delimiter.is_none() {
        anyhow::bail!("{operation} swap target must be a list");
    }
    if left_item_index >= container.children.len() || right_item_index >= container.children.len() {
        anyhow::bail!(
            "{operation} swap item index out of bounds: {} and {} for {} items",
            left_item_index,
            right_item_index,
            container.children.len()
        );
    }
    if left_item_index < protected_prefix_count || right_item_index < protected_prefix_count {
        anyhow::bail!("{operation} cannot swap protected list prefix items");
    }

    let start = container.span.start().get();
    let end = container.span.end().get();
    let open = &input[start..start + 1];
    let close = &input[end - 1..end];
    let mut items = container
        .children
        .iter()
        .map(|child| child.span.slice(input).to_owned())
        .collect::<Vec<_>>();
    items.swap(left_item_index, right_item_index);

    let replacement = if items.is_empty() {
        format!("{open}{close}")
    } else {
        format!("{open}{}{close}", items.join(" "))
    };
    Ok((container.span, replacement))
}

pub(super) fn reorder_list_items_edit(
    input: &str,
    container: &ExpressionView,
    protected_prefix_count: usize,
    new_relative_order: &[usize],
    operation: &str,
) -> Result<SpanEdit> {
    if container.kind != ExpressionKind::List || container.delimiter.is_none() {
        anyhow::bail!("{operation} reorder target must be a list");
    }
    if protected_prefix_count > container.children.len() {
        anyhow::bail!("{operation} protected prefix is out of bounds");
    }

    let item_count = container.children.len() - protected_prefix_count;
    if new_relative_order.len() != item_count {
        anyhow::bail!(
            "{operation} new order has {} items but target has {} reorderable items",
            new_relative_order.len(),
            item_count
        );
    }

    let mut seen = vec![false; item_count];
    for &index in new_relative_order {
        if index >= item_count {
            anyhow::bail!("{operation} new order index {index} is out of bounds");
        }
        if seen[index] {
            anyhow::bail!("{operation} new order contains duplicate index {index}");
        }
        seen[index] = true;
    }

    let start = container.span.start().get();
    let end = container.span.end().get();
    let open = &input[start..start + 1];
    let close = &input[end - 1..end];
    let mut items = container.children[..protected_prefix_count]
        .iter()
        .map(|child| child.span.slice(input).to_owned())
        .collect::<Vec<_>>();
    let reorderable_items = container.children[protected_prefix_count..]
        .iter()
        .map(|child| child.span.slice(input).to_owned())
        .collect::<Vec<_>>();
    items.extend(
        new_relative_order
            .iter()
            .map(|&index| reorderable_items[index].clone()),
    );

    let replacement = if items.is_empty() {
        format!("{open}{close}")
    } else {
        format!("{open}{}{close}", items.join(" "))
    };
    Ok((container.span, replacement))
}

pub(super) fn apply_byte_span_edits(input: &str, mut edits: Vec<SpanEdit>) -> Result<String> {
    edits.sort_by_key(|(span, _)| span.start());
    ensure_non_overlapping_spans(edits.iter().map(|(span, _)| *span))?;

    let mut output = input.to_owned();
    for (span, replacement) in edits.into_iter().rev() {
        output.replace_range(span.as_range(), &replacement);
    }
    Ok(output)
}

pub(super) fn ensure_non_overlapping_spans(
    spans: impl IntoIterator<Item = ByteSpan>,
) -> Result<()> {
    let mut previous_end = None;
    for span in spans {
        let start = span.start().get();
        let end = span.end().get();
        if let Some(previous_end) = previous_end {
            if start < previous_end {
                anyhow::bail!("refusing overlapping rewrite spans");
            }
        }
        previous_end = Some(end);
    }
    Ok(())
}

pub(super) fn spans_overlap(left: ByteSpan, right: ByteSpan) -> bool {
    left.start().get() < right.end().get() && right.start().get() < left.end().get()
}

pub(super) fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

pub(super) fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

pub(super) fn list_head(view: &ExpressionView) -> Option<&str> {
    if view.kind != ExpressionKind::List {
        return None;
    }
    atom_child(view, 0)
}
