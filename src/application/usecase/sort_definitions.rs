//! Use-case planner for sorting contiguous top-level definitions.

mod collect;
mod ordering;
mod rewrite;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

use anyhow::Result;

use crate::domain::sexpr::{Path, SyntaxTree};

use collect::collect_sortable_blocks;
use ordering::sorted_entry_positions;
use rewrite::apply_replacements;
use types::BlockReplacement;

pub use types::{
    SortDefinitionsItem, SortDefinitionsPlan, SortDefinitionsRequest, SortDefinitionsStrategy,
};

/// Separator used when the block's original first entry (which has no
/// leading trivia of its own) is reordered away from the front.
const DEFAULT_ENTRY_SEPARATOR: &str = "\n\n";

pub fn plan_sort_definitions(request: SortDefinitionsRequest<'_>) -> Result<SortDefinitionsPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    let blocks = collect_sortable_blocks(request.input, &tree, request.dialect)?;
    let mut replacements = Vec::new();
    let mut items = Vec::new();

    for block in blocks {
        let sorted_positions = sorted_entry_positions(&block.entries, request.strategy);
        let original_positions = (0..block.entries.len()).collect::<Vec<_>>();
        if sorted_positions == original_positions {
            continue;
        }

        let mut replacement = String::new();
        for (target_offset, source_position) in sorted_positions.iter().enumerate() {
            let entry = &block.entries[*source_position];
            // The block's original first entry carries no leading trivia of
            // its own. If it lands anywhere but the front after reordering,
            // it needs a separator so it doesn't glue onto the previous
            // entry's closing delimiter.
            if target_offset > 0 && !entry.has_leading_trivia {
                replacement.push_str(DEFAULT_ENTRY_SEPARATOR);
            }
            replacement.push_str(&entry.form_text);

            let target_index = block.entries[target_offset].item.source_index;
            let mut item = entry.item.clone();
            item.target_index = target_index;
            item.new_path = Path::root_child(target_index);
            items.push(item);
        }
        // When some other entry displaces the block's original first entry
        // from the front, that entry's own leading trivia (a blank run,
        // possibly with a comment) ends up at the top of the region, where
        // there is nothing left for it to separate from. Drop just the
        // blank-line run while leaving a leading comment in place.
        let replacement = if sorted_positions[0] == 0 {
            replacement
        } else {
            strip_leading_blank_lines(&replacement)
        };

        replacements.push(BlockReplacement {
            start: block.start,
            end: block.end,
            text: replacement,
        });
    }

    let rewritten = apply_replacements(request.input, &replacements);
    SyntaxTree::parse(&rewritten)?;
    let changed = rewritten != request.input;

    Ok(SortDefinitionsPlan {
        file: request.file,
        dialect: request.dialect,
        strategy: request.strategy,
        items,
        rewritten,
        changed,
        written: request.write && changed,
    })
}

/// Drops leading lines that are empty or all-whitespace, returning the
/// remainder starting at the first line with content (a comment or a form).
/// A comment's own indentation is left untouched.
///
/// A leading newline is always a slot-boundary marker (the newline that used
/// to end the previous definition's line), not a blank line by itself, so it
/// is dropped unconditionally before the remainder is scanned for genuine
/// blank lines.
fn strip_leading_blank_lines(text: &str) -> String {
    let text = text.strip_prefix('\n').unwrap_or(text);
    let bytes = text.as_bytes();
    let mut cursor = 0;
    loop {
        let line_end = bytes[cursor..]
            .iter()
            .position(|&byte| byte == b'\n')
            .map(|offset| cursor + offset);
        match line_end {
            Some(end) if text[cursor..end].trim().is_empty() => cursor = end + 1,
            _ => break,
        }
    }
    text[cursor..].to_owned()
}
