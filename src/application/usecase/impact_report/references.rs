use std::collections::BTreeSet;
use std::path::Path as FsPath;

use crate::application::usecase::signature_report::SignatureCallItem;
use crate::domain::common_lisp::common_lisp_symbol_name_eq;
use crate::domain::dialect::Dialect;
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
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
    dialect: Dialect,
    tree: &SyntaxTree,
    symbol: &SymbolName,
) -> Vec<AtomOccurrence> {
    if dialect == Dialect::CommonLisp {
        return matching_common_lisp_symbol_occurrences(tree, symbol);
    }

    tree.atom_occurrences()
        .into_iter()
        .filter(|occurrence| common_lisp_symbol_name_eq(&occurrence.text, symbol.as_str()))
        .collect()
}

fn matching_common_lisp_symbol_occurrences(
    tree: &SyntaxTree,
    symbol: &SymbolName,
) -> Vec<AtomOccurrence> {
    let mut spans = Vec::new();
    collect_unshadowed_symbol_references(&tree.root_view(), symbol, "", &mut spans);
    let matched_spans = spans
        .into_iter()
        .map(|span| (span.start().get(), span.end().get()))
        .collect::<BTreeSet<_>>();

    tree.atom_occurrences()
        .into_iter()
        .filter(|occurrence| {
            common_lisp_symbol_name_eq(&occurrence.text, symbol.as_str())
                && matched_spans
                    .contains(&(occurrence.span.start().get(), occurrence.span.end().get()))
        })
        .collect()
}

pub(super) fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start().get() <= inner.start().get() && inner.end().get() <= outer.end().get()
}
