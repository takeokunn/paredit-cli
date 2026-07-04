use std::collections::BTreeMap;

use crate::application::usecase::form_report::types::FormSymbolReport;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SymbolAccumulator {
    count: usize,
    first_span: ByteSpan,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct FormStats {
    pub(super) atom_count: usize,
    pub(super) list_count: usize,
    pub(super) max_depth: usize,
    symbols: BTreeMap<String, SymbolAccumulator>,
}

impl FormStats {
    pub(super) fn collect(view: &ExpressionView) -> Self {
        let mut stats = Self::default();
        collect_stats(view, 0, &mut stats);
        stats
    }

    pub(super) fn into_symbols(self) -> Vec<FormSymbolReport> {
        self.symbols
            .into_iter()
            .map(|(symbol, accumulator)| FormSymbolReport {
                symbol,
                count: accumulator.count,
                first_span: accumulator.first_span,
            })
            .collect()
    }
}

fn collect_stats(view: &ExpressionView, depth: usize, stats: &mut FormStats) {
    stats.max_depth = stats.max_depth.max(depth);

    match view.kind {
        ExpressionKind::Root | ExpressionKind::List => {
            stats.list_count += 1;
            for child in &view.children {
                collect_stats(child, depth + 1, stats);
            }
        }
        ExpressionKind::Atom => {
            stats.atom_count += 1;
            if let Some(text) = view
                .text
                .as_deref()
                .filter(|text| is_reportable_symbol(text))
            {
                let entry = stats
                    .symbols
                    .entry(text.to_owned())
                    .or_insert(SymbolAccumulator {
                        count: 0,
                        first_span: view.span,
                    });
                entry.count += 1;
            }
        }
    }
}

fn is_reportable_symbol(text: &str) -> bool {
    !text.is_empty() && !text.starts_with('"')
}
