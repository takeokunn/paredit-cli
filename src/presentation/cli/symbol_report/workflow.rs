use anyhow::Result;

use crate::domain::sexpr::ByteSpan;
use crate::presentation::cli::shared::{matching_symbol_occurrences, read_input_dialect_and_tree};

use super::args::{SymbolQueryArgs, SymbolReportArgs};
use super::render::{print_symbol_occurrences, print_symbol_report};
use super::types::{SymbolOccurrenceContext, SymbolReportFile, SymbolReportOccurrence};

pub(in crate::presentation::cli) fn find_symbol(args: SymbolQueryArgs) -> Result<()> {
    let (_, dialect, tree) = read_input_dialect_and_tree(args.file, args.dialect)?;
    print_symbol_occurrences(&tree, dialect, &args.symbol, args.output)
}

pub(in crate::presentation::cli) fn symbol_report(args: SymbolReportArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let (_, dialect, tree) = read_input_dialect_and_tree(Some(file.clone()), args.dialect)?;
        let outline = tree.outline(|head| dialect.is_definition_head(head));
        let occurrences = matching_symbol_occurrences(&tree, &args.symbol)
            .into_iter()
            .map(|occurrence| SymbolReportOccurrence {
                path: occurrence.path.to_string(),
                span: occurrence.span,
                context: outline
                    .iter()
                    .filter(|entry| span_contains(entry.span, occurrence.span))
                    .min_by_key(|entry| entry.span.end().get() - entry.span.start().get())
                    .map(|entry| SymbolOccurrenceContext {
                        path: entry.path.to_string(),
                        span: entry.span,
                        head: entry.head.clone(),
                        definition_like: entry.definition_like,
                    }),
            })
            .collect::<Vec<_>>();

        reports.push(SymbolReportFile {
            path: file.clone(),
            dialect,
            occurrences,
        });
    }

    print_symbol_report(&reports, &args.symbol, args.output)
}

fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start().get() <= inner.start().get() && inner.end().get() <= outer.end().get()
}
