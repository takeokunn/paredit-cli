use std::collections::HashSet;

use crate::application::usecase::remove_unused_definition::{
    collect_function_quote_references, collect_package_form_spans, collect_quoted_data_references,
    collect_reference_needles,
};
use crate::domain::common_lisp::common_lisp_symbol_reference_needle;
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

    // See `collect_reference_needles`: a file whose atom set lacks a
    // symbol's normalized name cannot reference it, so the three per-symbol
    // tree walks below can skip that file after one hash lookup.
    let atom_needles: Vec<HashSet<String>> = views
        .iter()
        .map(|view| {
            let mut needles = HashSet::new();
            if let Some(view) = view {
                collect_reference_needles(view, &mut needles);
            }
            needles
        })
        .collect();

    // Each file's report depends only on the shared read-only parses above,
    // so the per-definition reference scans (the dominant cost on large
    // workspaces) fan out across workers; results are reassembled by index
    // to keep output order deterministic.
    let worker_count = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1)
        .clamp(1, files.len().max(1));
    let mut ordered: Vec<Option<UnusedDefinitionFile>> = (0..files.len()).map(|_| None).collect();
    std::thread::scope(|scope| {
        let views = &views;
        let package_form_spans = &package_form_spans;
        let atom_needles = &atom_needles;
        let handles: Vec<_> = (0..worker_count)
            .map(|worker| {
                scope.spawn(move || {
                    files
                        .iter()
                        .enumerate()
                        .skip(worker)
                        .step_by(worker_count)
                        .map(|(file_index, file)| {
                            (
                                file_index,
                                file_unused_definition_report(
                                    files,
                                    views,
                                    package_form_spans,
                                    atom_needles,
                                    file_index,
                                    file,
                                ),
                            )
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for handle in handles {
            for (file_index, report) in handle
                .join()
                .expect("unused-definition reference worker thread panicked")
            {
                ordered[file_index] = Some(report);
            }
        }
    });
    ordered.into_iter().flatten().collect()
}

fn file_unused_definition_report(
    files: &[ParsedDefinitionFile],
    views: &[Option<crate::domain::sexpr::ExpressionView>],
    package_form_spans: &[Vec<ByteSpan>],
    atom_needles: &[HashSet<String>],
    file_index: usize,
    file: &ParsedDefinitionFile,
) -> UnusedDefinitionFile {
    UnusedDefinitionFile {
        path: file.path.clone(),
        dialect: file.dialect,
        package: file.package.clone(),
        definitions: file
            .definitions
            .iter()
            .filter_map(|definition| {
                let name = definition.name.as_ref()?;
                let symbol = SymbolName::new(name.clone()).ok()?;
                let needle = common_lisp_symbol_reference_needle(symbol.as_str());
                let references = files
                    .iter()
                    .enumerate()
                    .flat_map(|(other_index, other)| {
                        let mut spans = Vec::new();
                        if let Some(view) = views[other_index]
                            .as_ref()
                            .filter(|_| atom_needles[other_index].contains(&needle))
                        {
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
                                .any(|package| package.contains_span(*span))
                        });

                        spans
                            .into_iter()
                            .filter(move |span| {
                                !(other_index == file_index && definition.span.contains_span(*span))
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
    }
}
