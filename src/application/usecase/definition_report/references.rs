use crate::domain::common_lisp::common_lisp_symbol_name_eq;
use crate::domain::sexpr::ByteSpan;

use super::types::{
    DefinitionReference, ParsedDefinitionFile, UnusedDefinitionFile, UnusedDefinitionItem,
};

pub fn collect_unused_definition_candidates(
    files: &[ParsedDefinitionFile],
) -> Vec<UnusedDefinitionFile> {
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
                    let references = files
                        .iter()
                        .enumerate()
                        .flat_map(|(other_index, other)| {
                            other
                                .atoms
                                .iter()
                                .filter(move |occurrence| {
                                    common_lisp_symbol_name_eq(&occurrence.text, name)
                                })
                                .filter(move |occurrence| {
                                    !(other_index == file_index
                                        && span_contains(definition.span, occurrence.span))
                                })
                                .map(move |occurrence| DefinitionReference {
                                    file_index: other_index,
                                    path: occurrence.path.to_string(),
                                    span: occurrence.span,
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

fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}
