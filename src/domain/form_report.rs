use std::collections::BTreeMap;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormReportRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub target: ExpressionView,
    pub include_source: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormReport {
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub span: ByteSpan,
    pub kind: FormKind,
    pub delimiter: Option<Delimiter>,
    pub head: Option<String>,
    pub definition_like: bool,
    pub child_count: usize,
    pub atom_count: usize,
    pub list_count: usize,
    pub max_depth: usize,
    pub symbols: Vec<FormSymbolReport>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormKind {
    Atom,
    List,
}

impl FormKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Atom => "atom",
            Self::List => "list",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSymbolReport {
    pub symbol: String,
    pub count: usize,
    pub first_span: ByteSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SymbolAccumulator {
    count: usize,
    first_span: ByteSpan,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct FormStats {
    atom_count: usize,
    list_count: usize,
    max_depth: usize,
    symbols: BTreeMap<String, SymbolAccumulator>,
}

impl FormStats {
    fn collect(view: &ExpressionView) -> Self {
        let mut stats = Self::default();
        collect_stats(view, 0, &mut stats);
        stats
    }

    fn into_symbols(self) -> Vec<FormSymbolReport> {
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

pub fn build_form_report(request: FormReportRequest<'_>) -> FormReport {
    let stats = FormStats::collect(&request.target);
    let head = expression_head(&request.target).map(ToOwned::to_owned);
    let definition_like = head
        .as_deref()
        .is_some_and(|head| request.dialect.is_definition_head(head));
    let source = request
        .include_source
        .then(|| request.target.span.slice(request.input).to_owned());

    FormReport {
        dialect: request.dialect,
        path: request.path,
        span: request.target.span,
        kind: form_kind(&request.target),
        delimiter: request.target.delimiter,
        head,
        definition_like,
        child_count: request.target.children.len(),
        atom_count: stats.atom_count,
        list_count: stats.list_count,
        max_depth: stats.max_depth,
        symbols: stats.into_symbols(),
        source,
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
                .filter(|text| !text.is_empty() && !text.starts_with('"'))
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

fn form_kind(view: &ExpressionView) -> FormKind {
    match view.kind {
        ExpressionKind::Atom => FormKind::Atom,
        ExpressionKind::List | ExpressionKind::Root => FormKind::List,
    }
}

fn expression_head(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::List)
        .then(|| {
            view.children
                .first()
                .and_then(|child| child.text.as_deref())
        })
        .flatten()
}
