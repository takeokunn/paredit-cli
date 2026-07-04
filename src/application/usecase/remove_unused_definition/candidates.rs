use crate::application::usecase::remove_unused_definition::types::{
    RemoveUnusedDefinitionInputFile, UnusedDefinitionDefinition,
};
use crate::domain::sexpr::ByteSpan;

#[derive(Debug)]
pub(super) struct UnusedDefinitionItem {
    pub(super) definition: UnusedDefinitionDefinition,
    pub(super) references: Vec<DefinitionReference>,
}

#[derive(Debug)]
pub(super) struct UnusedDefinitionFile {
    pub(super) definitions: Vec<UnusedDefinitionItem>,
}

#[derive(Debug)]
pub(super) struct DefinitionReference;

pub(super) fn collect_unused_definition_candidates(
    files: &[RemoveUnusedDefinitionInputFile],
) -> Vec<UnusedDefinitionFile> {
    files
        .iter()
        .enumerate()
        .map(|(file_index, file)| UnusedDefinitionFile {
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
                                .filter(move |occurrence| occurrence.text == *name)
                                .filter(move |occurrence| {
                                    !(other_index == file_index
                                        && span_contains(definition.span, occurrence.span))
                                })
                                .map(|_occurrence| DefinitionReference)
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

fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start().get() <= inner.start().get() && inner.end().get() <= outer.end().get()
}
