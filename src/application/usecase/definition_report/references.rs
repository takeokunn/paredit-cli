use crate::application::usecase::remove_unused_definition::{
    collect_function_quote_references, collect_package_form_spans, collect_quoted_data_references,
};
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{ByteSpan, SymbolName, SyntaxTree};

use super::types::{
    DefinitionReference, ParsedDefinitionFile, UnusedDefinitionFile, UnusedDefinitionItem,
};

pub fn collect_unused_definition_candidates(
    files: &[ParsedDefinitionFile],
) -> Vec<UnusedDefinitionFile> {
    // Scope-aware value-namespace collection (`collect_unshadowed_symbol_references`)
    // is the primary scan, supplemented by function-namespace (`#'name`) and
    // quoted-data-table traversals. This must stay in sync with
    // `remove_unused_definition::candidates`, which the `remove-unused-definitions`
    // command uses for the same "is this definition still referenced?"
    // question — otherwise the two commands would disagree on whether a
    // same-named local binding elsewhere shadows or shows a real reference.
    let views: Vec<_> = files
        .iter()
        .map(|file| {
            SyntaxTree::parse(&file.text)
                .ok()
                .map(|tree| tree.root_view())
        })
        .collect();

    let package_form_spans: Vec<Vec<ByteSpan>> = files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let mut spans = Vec::new();
            if let Some(view) = &views[index] {
                collect_package_form_spans(file.dialect, view, &mut spans);
            }
            spans
        })
        .collect();

    files
        .iter()
        .enumerate()
        .map(|(file_index, file)| UnusedDefinitionFile {
            path: file.path.clone(),
            dialect: file.dialect,
            package: file.package.clone(),
            definitions: file
                .definitions
                .iter()
                .filter_map(|definition| {
                    let name = definition.name.as_ref()?;
                    let symbol = SymbolName::new(name.clone()).ok()?;
                    let references = files
                        .iter()
                        .enumerate()
                        .flat_map(|(other_index, other)| {
                            let mut spans = Vec::new();
                            if let Some(view) = &views[other_index] {
                                collect_unshadowed_symbol_references(
                                    other.dialect,
                                    view,
                                    &symbol,
                                    &other.text,
                                    &mut spans,
                                );
                                collect_function_quote_references(
                                    other.dialect,
                                    view,
                                    &symbol,
                                    &mut spans,
                                );
                                collect_quoted_data_references(
                                    other.dialect,
                                    view,
                                    &symbol,
                                    &mut spans,
                                );
                            }

                            let other_package_spans = &package_form_spans[other_index];
                            spans.retain(|span| {
                                !other_package_spans
                                    .iter()
                                    .any(|package| span_contains(*package, *span))
                            });

                            spans
                                .into_iter()
                                .filter(move |span| {
                                    !(other_index == file_index
                                        && span_contains(definition.span, *span))
                                })
                                .map(move |span| DefinitionReference {
                                    file_index: other_index,
                                    path: String::new(),
                                    span,
                                })
                        })
                        .collect();

                    Some(UnusedDefinitionItem {
                        definition: definition.clone(),
                        references,
                    })
                })
                .collect(),
        })
        .collect()
}

pub fn unused_definition_candidate_count(reports: &[UnusedDefinitionFile]) -> usize {
    reports
        .iter()
        .flat_map(|report| &report.definitions)
        .filter(|item| item.references.is_empty())
        .count()
}

/// Counts only candidates whose category `DefinitionCategory::is_bulk_removable`
/// accepts. `Test`, `Package`, `Struct`, and the other protected categories are
/// normally unreferenced by symbol from other code by design (an `ert-deftest`
/// is invoked by name from a test runner, a `provide` form by the module
/// loader, ...), so counting them toward "unused" would make `--fail-on-unused`
/// trip on a healthy codebase's ordinary test suite.
pub fn unused_definition_actionable_candidate_count(reports: &[UnusedDefinitionFile]) -> usize {
    reports
        .iter()
        .flat_map(|report| &report.definitions)
        .filter(|item| item.references.is_empty() && item.definition.category.is_bulk_removable())
        .count()
}

fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}
