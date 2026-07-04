use anyhow::Result;

use crate::domain::definition::definition_name_child_index;
use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Selection, SymbolName, SyntaxTree,
};

use super::RenameTarget;

pub(super) fn select_rename_target<'a>(
    tree: &'a SyntaxTree,
    target: &RenameTarget,
) -> Result<Selection<'a>> {
    match target {
        RenameTarget::Path(path) => tree.select_path(path),
        RenameTarget::Offset(offset) => tree.select_at(*offset),
    }
}

pub(super) fn collect_symbol_atom_spans(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
) {
    if atom_text(view).is_some_and(|text| text == symbol.as_str()) {
        output.push(view.span);
        return;
    }
    for child in &view.children {
        collect_symbol_atom_spans(child, symbol, output);
    }
}

pub(super) fn apply_byte_span_edits(
    input: &str,
    mut edits: Vec<(ByteSpan, String)>,
) -> Result<String> {
    edits.sort_by_key(|(span, _)| span.start());
    ensure_non_overlapping_spans(edits.iter().map(|(span, _)| *span))?;

    let mut output = input.to_owned();
    for (span, replacement) in edits.into_iter().rev() {
        output.replace_range(span.as_range(), &replacement);
    }
    Ok(output)
}

fn ensure_non_overlapping_spans(spans: impl IntoIterator<Item = ByteSpan>) -> Result<()> {
    let mut previous: Option<ByteSpan> = None;
    for span in spans {
        if let Some(previous) = previous {
            if previous.end().get() > span.start().get() {
                anyhow::bail!(
                    "overlapping edits at {}..{} and {}..{}",
                    previous.start().get(),
                    previous.end().get(),
                    span.start().get(),
                    span.end().get()
                );
            }
        }
        previous = Some(span);
    }
    Ok(())
}

pub(super) fn list_head(view: &ExpressionView) -> Option<&str> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return None;
    }

    atom_child(view, 0)
}

pub(super) fn definition_name<'a>(view: &'a ExpressionView, head: &str) -> Option<&'a str> {
    definition_name_child_index(head).and_then(|index| atom_child(view, index))
}

fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

pub(super) fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

pub(super) fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start().get() <= inner.start().get() && inner.end().get() <= outer.end().get()
}
