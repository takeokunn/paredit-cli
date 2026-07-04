use std::collections::BTreeSet;
use std::path::Path as FsPath;

use crate::application::signature_report::SignatureCallItem;
use crate::domain::sexpr::{AtomOccurrence, ByteSpan, SymbolName, SyntaxTree};

use super::types::{ImpactDefinitionItem, ImpactSymbolOccurrence};

pub(super) fn count_non_call_references(
    path: &FsPath,
    references: &[ImpactSymbolOccurrence],
    definitions: &[ImpactDefinitionItem],
    calls: &[SignatureCallItem],
) -> usize {
    let mut excluded = BTreeSet::new();

    for reference in references {
        let reference_span = reference.span;
        if definitions
            .iter()
            .any(|definition| span_contains(definition.span, reference_span))
        {
            excluded.insert(symbol_occurrence_key(path, reference_span));
            continue;
        }
        if calls
            .iter()
            .any(|call| span_contains(call.call.span, reference_span))
        {
            excluded.insert(symbol_occurrence_key(path, reference_span));
        }
    }

    references
        .iter()
        .filter(|reference| !excluded.contains(&symbol_occurrence_key(path, reference.span)))
        .count()
}

fn symbol_occurrence_key(path: &FsPath, span: ByteSpan) -> (String, usize, usize) {
    (
        path.display().to_string(),
        span.start().get(),
        span.end().get(),
    )
}

pub(super) fn matching_symbol_occurrences(
    tree: &SyntaxTree,
    symbol: &SymbolName,
) -> Vec<AtomOccurrence> {
    tree.atom_occurrences()
        .into_iter()
        .filter(|occurrence| occurrence.text == symbol.as_str())
        .collect()
}

pub(super) fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start().get() <= inner.start().get() && inner.end().get() <= outer.end().get()
}
