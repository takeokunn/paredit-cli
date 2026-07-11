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
            replacement.push_str(&entry.form_text);
            if let Some(separator) = block.separators.get(target_offset) {
                replacement.push_str(separator);
            }

            let target_index = block.entries[target_offset].item.source_index;
            let mut item = entry.item.clone();
            item.target_index = target_index;
            item.new_path = Path::root_child(target_index);
            items.push(item);
        }

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
