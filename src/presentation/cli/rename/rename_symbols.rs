use std::fs;

use anyhow::{Context, Result};

use super::super::{detect_dialect, read_input};
use super::args::RenameSymbolsArgs;
use super::render::symbols::print_rename_symbols_report;
use super::types::RenameFileReport;
use crate::domain::sexpr::SyntaxTree;
use crate::presentation::cli::shared::{apply_byte_span_edits, matching_symbol_occurrences};

pub(in crate::presentation::cli) fn rename_symbols(args: RenameSymbolsArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());
    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        let occurrences = matching_symbol_occurrences(&tree, &args.from)
            .into_iter()
            .map(|occurrence| occurrence.span)
            .collect::<Vec<_>>();
        let edits = occurrences
            .iter()
            .map(|span| (*span, args.to.as_str().to_owned()))
            .collect::<Vec<_>>();
        let rewritten = apply_byte_span_edits(&input.text, edits)?;
        let changed = rewritten != input.text;
        let written = args.write && changed;
        if written {
            fs::write(file, &rewritten)
                .with_context(|| format!("failed to write {}", file.display()))?;
        }
        reports.push(RenameFileReport {
            path: file.clone(),
            dialect,
            occurrences,
            changed,
            written,
            rewritten,
        });
    }

    print_rename_symbols_report(&reports, &args.from, &args.to, args.write, args.output)
}
