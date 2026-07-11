use anyhow::{Result, anyhow};

use super::tree::{Node, NodeKind, Selection, SyntaxTree};
use super::types::{ByteOffset, ByteSpan, NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edit;

impl Edit {
    pub fn replace(input: &str, selection: Selection<'_>, replacement: &str) -> String {
        replace_span(input, selection.span(), replacement)
    }

    pub fn kill(input: &str, _tree: &SyntaxTree, selection: Selection<'_>) -> Result<String> {
        let span = expand_removal(input, selection.span());
        Ok(replace_span(input, span, ""))
    }

    pub fn wrap(input: &str, _tree: &SyntaxTree, selection: Selection<'_>) -> Result<String> {
        Ok(Self::replace(
            input,
            selection,
            &format!("({})", selection.text(input)),
        ))
    }

    pub fn splice(input: &str, _tree: &SyntaxTree, selection: Selection<'_>) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let (open, close) = list_delimiter_offsets(node)?;
        let mut output = String::with_capacity(input.len().saturating_sub(2));
        output.push_str(&input[..open]);
        output.push_str(&input[open + 1..close]);
        output.push_str(&input[close + 1..]);
        Ok(output)
    }

    pub fn raise(input: &str, _tree: &SyntaxTree, selection: Selection<'_>) -> Result<String> {
        let node = selection.node();
        let parent_id = node
            .parent
            .ok_or_else(|| anyhow!("selected node has no parent"))?;
        let parent = selection.tree.node(parent_id);
        if parent.kind == NodeKind::Root {
            anyhow::bail!("cannot raise a top-level expression");
        }
        Ok(replace_span(input, parent.span, selection.text(input)))
    }

    pub fn slurp_forward(
        input: &str,
        tree: &SyntaxTree,
        selection: Selection<'_>,
    ) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let sibling = next_sibling(tree, selection.node_id)
            .ok_or_else(|| anyhow!("selected list has no next sibling to slurp"))?;
        let (_, close) = list_delimiter_offsets(node)?;
        let insertion = format!(" {}", tree.node(sibling).span.slice(input));
        let removal = expand_removal(input, tree.node(sibling).span);
        Ok(remove_then_insert(
            input,
            removal,
            ByteOffset::new(close),
            &insertion,
        ))
    }

    pub fn slurp_backward(
        input: &str,
        tree: &SyntaxTree,
        selection: Selection<'_>,
    ) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let sibling = previous_sibling(tree, selection.node_id)
            .ok_or_else(|| anyhow!("selected list has no previous sibling to slurp"))?;
        let (open, _) = list_delimiter_offsets(node)?;
        let open = open + 1;
        let insertion = format!("{} ", tree.node(sibling).span.slice(input));
        let removal = expand_removal(input, tree.node(sibling).span);
        Ok(remove_then_insert(
            input,
            removal,
            ByteOffset::new(open),
            &insertion,
        ))
    }

    pub fn barf_forward(
        input: &str,
        tree: &SyntaxTree,
        selection: Selection<'_>,
    ) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let child = *node
            .children
            .last()
            .ok_or_else(|| anyhow!("cannot barf from an empty list"))?;
        let (_, close) = list_delimiter_offsets(node)?;
        let child_span = tree.node(child).span;
        let insertion = format!(" {}", child_span.slice(input));
        let removal = expand_removal(input, child_span);
        Ok(remove_then_insert(
            input,
            removal,
            ByteOffset::new(close + 1),
            &insertion,
        ))
    }

    pub fn barf_backward(
        input: &str,
        tree: &SyntaxTree,
        selection: Selection<'_>,
    ) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let child = *node
            .children
            .first()
            .ok_or_else(|| anyhow!("cannot barf from an empty list"))?;
        let open = node
            .open
            .ok_or_else(|| anyhow!("selected list is missing an opening delimiter"))?;
        let child_span = tree.node(child).span;
        let insertion = format!("{} ", child_span.slice(input));
        let removal = expand_removal(input, child_span);
        Ok(remove_then_insert(input, removal, open, &insertion))
    }
}

fn ensure_list(node: &Node) -> Result<()> {
    if node.kind != NodeKind::List {
        anyhow::bail!("operation requires a list expression");
    }
    Ok(())
}

fn list_delimiter_offsets(node: &Node) -> Result<(usize, usize)> {
    let open = node
        .open
        .ok_or_else(|| anyhow!("selected list is missing an opening delimiter"))?;
    let close = node
        .close
        .ok_or_else(|| anyhow!("selected list is missing a closing delimiter"))?;
    Ok((open.get(), close.get()))
}

fn next_sibling(tree: &SyntaxTree, node_id: NodeId) -> Option<NodeId> {
    let parent = tree.node(node_id).parent?;
    let siblings = &tree.node(parent).children;
    let position = siblings.iter().position(|id| *id == node_id)?;
    siblings.get(position + 1).copied()
}

fn previous_sibling(tree: &SyntaxTree, node_id: NodeId) -> Option<NodeId> {
    let parent = tree.node(node_id).parent?;
    let siblings = &tree.node(parent).children;
    let position = siblings.iter().position(|id| *id == node_id)?;
    position
        .checked_sub(1)
        .and_then(|previous| siblings.get(previous).copied())
}

fn replace_span(input: &str, span: ByteSpan, replacement: &str) -> String {
    let mut output = String::with_capacity(input.len() + replacement.len());
    output.push_str(&input[..span.start().get()]);
    output.push_str(replacement);
    output.push_str(&input[span.end().get()..]);
    output
}

fn expand_removal(input: &str, span: ByteSpan) -> ByteSpan {
    let bytes = input.as_bytes();
    let mut start = span.start().get();
    let mut end = span.end().get();
    if end < bytes.len() && bytes[end].is_ascii_whitespace() {
        while end < bytes.len() && bytes[end].is_ascii_whitespace() {
            end += 1;
        }
    } else {
        while start > 0 && bytes[start - 1].is_ascii_whitespace() {
            start -= 1;
        }
    }
    ByteSpan::new(ByteOffset::new(start), ByteOffset::new(end))
}

fn remove_then_insert(
    input: &str,
    removal: ByteSpan,
    insertion_at: ByteOffset,
    insertion: &str,
) -> String {
    let adjusted_insertion_at = if insertion_at.get() > removal.end().get() {
        insertion_at.get() - removal.len()
    } else {
        insertion_at.get()
    };
    let removed = replace_span(input, removal, "");
    replace_span(
        &removed,
        ByteSpan::new(
            ByteOffset::new(adjusted_insertion_at),
            ByteOffset::new(adjusted_insertion_at),
        ),
        insertion,
    )
}
