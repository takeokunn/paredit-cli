use anyhow::{Context, Result};

use crate::application::usecase::remove_unused_definition::types::{
    RemoveUnusedDefinitionInputFile, UnusedDefinitionDefinition,
};
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{ByteSpan, SymbolName, SyntaxTree};

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
) -> Result<Vec<UnusedDefinitionFile>> {
    let parsed_files = files
        .iter()
        .map(|file| -> Result<_> {
            let tree = SyntaxTree::parse(&file.text)
                .with_context(|| format!("failed to parse {}", file.path.display()))?;
            Ok((file, tree.root_view()))
        })
        .collect::<Result<Vec<_>>>()?;

    files
        .iter()
        .enumerate()
        .map(|(file_index, file)| -> Result<_> {
            let definitions = file
                .definitions
                .iter()
                .filter_map(|definition| {
                    let name = definition.name.as_ref()?;
                    Some((definition, name))
                })
                .map(|(definition, name)| -> Result<_> {
                    let symbol = SymbolName::new(name.clone()).with_context(|| {
                        format!(
                            "remove-unused-definition found invalid symbol '{}' in {}",
                            name,
                            file.path.display()
                        )
                    })?;
                    let references = files
                        .iter()
                        .enumerate()
                        .flat_map(|(other_index, other)| {
                            let (_, other_view) = &parsed_files[other_index];
                            let mut spans = Vec::new();
                            collect_unshadowed_symbol_references(
                                other_view,
                                &symbol,
                                &other.text,
                                &mut spans,
                            );
                            spans
                                .into_iter()
                                .filter(move |span| {
                                    !(other_index == file_index
                                        && span_contains(definition.span, *span))
                                })
                                .map(|_span| DefinitionReference)
                        })
                        .collect();

                    Ok(UnusedDefinitionItem {
                        definition: definition.clone(),
                        references,
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(UnusedDefinitionFile { definitions })
        })
        .collect()
}

fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start().get() <= inner.start().get() && inner.end().get() <= outer.end().get()
}
