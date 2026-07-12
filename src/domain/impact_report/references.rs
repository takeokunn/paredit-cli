use std::collections::BTreeSet;
use std::path::Path as FsPath;

use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{AtomOccurrence, ByteSpan, SymbolName, SyntaxTree};
use crate::domain::signature_report::SignatureCallItem;

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
            .any(|definition| definition.span.contains_span(reference_span))
        {
            excluded.insert(symbol_occurrence_key(path, reference_span));
            continue;
        }
        if calls
            .iter()
            .any(|call| call.call.span.contains_span(reference_span))
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
    dialect: Dialect,
    tree: &SyntaxTree,
    symbol: &SymbolName,
) -> Vec<AtomOccurrence> {
    let occurrences = tree.atom_occurrences();

    if dialect == Dialect::CommonLisp {
        return matching_common_lisp_symbol_occurrences(occurrences, tree, symbol);
    }

    occurrences
        .into_iter()
        .filter(|occurrence| common_lisp_symbol_reference_eq(&occurrence.text, symbol.as_str()))
        .collect()
}

fn matching_common_lisp_symbol_occurrences(
    occurrences: Vec<AtomOccurrence>,
    tree: &SyntaxTree,
    symbol: &SymbolName,
) -> Vec<AtomOccurrence> {
    let mut spans = Vec::new();
    collect_unshadowed_symbol_references(
        Dialect::CommonLisp,
        &tree.root_view(),
        symbol,
        "",
        &mut spans,
    );
    let matched_spans = spans
        .into_iter()
        .map(|span| (span.start().get(), span.end().get()))
        .collect::<BTreeSet<_>>();

    occurrences
        .into_iter()
        .filter(|occurrence| {
            common_lisp_symbol_reference_eq(&occurrence.text, symbol.as_str())
                && matched_spans
                    .contains(&(occurrence.span.start().get(), occurrence.span.end().get()))
        })
        .collect()
}
