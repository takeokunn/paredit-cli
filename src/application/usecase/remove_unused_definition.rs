use anyhow::{Context, Result};

use crate::domain::sexpr::SyntaxTree;

mod candidates;
mod policy;
mod rewrite;
#[cfg(test)]
mod tests;
mod types;

use candidates::collect_unused_definition_candidates;
use policy::{collect_exported_symbol_index, definition_is_bulk_removable, definition_is_exported};
use rewrite::{expand_definition_removal, replace_span};

pub use types::{
    PlannedDefinitionRemoval, RemoveUnusedDefinitionInputFile, RemoveUnusedDefinitionsFilePlan,
    RemoveUnusedDefinitionsPlan, RemoveUnusedDefinitionsRequest, SkippedDefinitionRemoval,
    SkippedDefinitionRemovalReason, UnusedDefinitionDefinition,
};

pub fn plan_remove_unused_definitions(
    request: RemoveUnusedDefinitionsRequest,
) -> Result<RemoveUnusedDefinitionsPlan> {
    let exported_symbols = collect_exported_symbol_index(&request.package_definitions);
    let unused_reports = collect_unused_definition_candidates(&request.files);
    let mut files = Vec::with_capacity(request.files.len());

    for (file, report) in request.files.iter().zip(&unused_reports) {
        let mut removals = Vec::new();
        let mut skipped = Vec::new();

        for item in report
            .definitions
            .iter()
            .filter(|item| item.references.is_empty())
        {
            if !request.include_exported
                && definition_is_exported(&item.definition, &exported_symbols)
            {
                skipped.push(SkippedDefinitionRemoval {
                    definition: item.definition.clone(),
                    reason: SkippedDefinitionRemovalReason::ExportedDefinition,
                });
            } else if request.include_protected
                || definition_is_bulk_removable(item.definition.category)
            {
                let definition_text = item.definition.span.slice(&file.text).to_owned();
                removals.push(PlannedDefinitionRemoval {
                    definition: item.definition.clone(),
                    definition_text,
                    removal_span: item.definition.span,
                });
            } else {
                skipped.push(SkippedDefinitionRemoval {
                    definition: item.definition.clone(),
                    reason: SkippedDefinitionRemovalReason::ProtectedDefinitionCategory,
                });
            }
        }

        removals.sort_by(|left, right| {
            right
                .definition
                .span
                .start()
                .get()
                .cmp(&left.definition.span.start().get())
        });

        let mut rewritten = file.text.clone();
        for removal in &mut removals {
            let expanded = expand_definition_removal(&rewritten, removal.definition.span);
            removal.removal_span = expanded;
            rewritten = replace_span(&rewritten, expanded, "");
        }

        SyntaxTree::parse(&rewritten).with_context(|| {
            format!(
                "file would become invalid after removing unused definitions: {}",
                file.path.display()
            )
        })?;

        let changed = rewritten != file.text;
        files.push(RemoveUnusedDefinitionsFilePlan {
            path: file.path.clone(),
            dialect: file.dialect,
            package: file.package.clone(),
            rewritten,
            changed,
            removals,
            skipped,
        });
    }

    let candidate_count = unused_reports
        .iter()
        .flat_map(|report| &report.definitions)
        .filter(|item| item.references.is_empty())
        .count();
    let removal_count = files.iter().map(|file| file.removals.len()).sum();
    let skipped_count = files.iter().map(|file| file.skipped.len()).sum();
    let changed = files.iter().any(|file| file.changed);

    Ok(RemoveUnusedDefinitionsPlan {
        files,
        candidate_count,
        removal_count,
        skipped_count,
        changed,
    })
}
