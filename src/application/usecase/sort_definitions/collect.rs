use anyhow::Result;

use crate::domain::definition::{DefinitionCategory, classify_definition_head};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SyntaxTree};

use super::syntax::{definition_name, list_head};
use super::types::{DefinitionBlock, DefinitionEntry, RawDefinition, SortDefinitionsItem};

pub(super) fn collect_sortable_blocks(
    input: &str,
    tree: &SyntaxTree,
    dialect: Dialect,
) -> Result<Vec<DefinitionBlock>> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    for index in 0..tree.root_children().len() {
        let path = Path::from_indexes(vec![index]);
        let selection = tree.select_path(&path)?;
        let view = selection.view();
        let Some(head) = list_head(&view) else {
            finish_block(input, &mut current, &mut blocks);
            continue;
        };
        let Some(category) = classify_definition_head(dialect, head) else {
            finish_block(input, &mut current, &mut blocks);
            continue;
        };

        if !is_sortable_category(category) {
            finish_block(input, &mut current, &mut blocks);
            continue;
        }

        let Some(name) = definition_name(&view, head).map(ToOwned::to_owned) else {
            finish_block(input, &mut current, &mut blocks);
            continue;
        };

        current.push(RawDefinition {
            path,
            span: selection.span(),
            head: head.to_owned(),
            name: Some(name),
            category,
            source_index: index,
        });
    }
    finish_block(input, &mut current, &mut blocks);

    Ok(blocks)
}

fn finish_block(input: &str, current: &mut Vec<RawDefinition>, blocks: &mut Vec<DefinitionBlock>) {
    if current.len() < 2 {
        current.clear();
        return;
    }

    let mut entries = Vec::with_capacity(current.len());
    let mut separators = Vec::with_capacity(current.len().saturating_sub(1));
    for index in 0..current.len() {
        let chunk_start = current[index].span.start().get();
        let chunk_end = current[index].span.end().get();
        entries.push(DefinitionEntry {
            item: SortDefinitionsItem {
                old_path: current[index].path.clone(),
                new_path: current[index].path.clone(),
                span: current[index].span,
                head: current[index].head.clone(),
                name: current[index].name.clone(),
                category: current[index].category,
                source_index: current[index].source_index,
                target_index: current[index].source_index,
            },
            form_text: input[chunk_start..chunk_end].to_owned(),
        });
        if let Some(next) = current.get(index + 1) {
            separators.push(input[chunk_end..next.span.start().get()].to_owned());
        }
    }

    blocks.push(DefinitionBlock {
        start: current
            .first()
            .expect("block has at least two definitions")
            .span
            .start()
            .get(),
        end: current
            .last()
            .expect("block has at least two definitions")
            .span
            .end()
            .get(),
        entries,
        separators,
    });
    current.clear();
}

fn is_sortable_category(category: DefinitionCategory) -> bool {
    !matches!(
        category,
        DefinitionCategory::Package | DefinitionCategory::System
    )
}
