use std::collections::BTreeMap;

use anyhow::Result;

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

#[derive(Debug, Default)]
struct FormStats {
    atom_count: usize,
    list_count: usize,
    max_depth: usize,
    symbols: BTreeMap<String, SymbolAccumulator>,
}

pub fn build_form_report(request: FormReportRequest<'_>) -> Result<FormReport> {
    let mut stats = FormStats::default();
    collect_stats(&request.target, 0, &mut stats);

    let head = expression_head(&request.target).map(ToOwned::to_owned);
    let definition_like = head
        .as_deref()
        .is_some_and(|head| request.dialect.is_definition_head(head));
    let source = request
        .include_source
        .then(|| request.target.span.slice(request.input).to_owned());

    Ok(FormReport {
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
        symbols: stats
            .symbols
            .into_iter()
            .map(|(symbol, accumulator)| FormSymbolReport {
                symbol,
                count: accumulator.count,
                first_span: accumulator.first_span,
            })
            .collect(),
        source,
    })
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::sexpr::SyntaxTree;

    fn report_for(input: &str, path: &str, dialect: Dialect) -> FormReport {
        let tree = SyntaxTree::parse(input).expect("valid input");
        let path = path.parse::<Path>().expect("valid path");
        let selection = tree.select_path(&path).expect("selection");
        build_form_report(FormReportRequest {
            input,
            dialect,
            path: Some(path),
            target: selection.view(),
            include_source: true,
        })
        .expect("report")
    }

    #[test]
    fn reports_definition_like_common_lisp_form() {
        let report = report_for("(defun add (x y) (+ x y))", "0", Dialect::CommonLisp);

        assert_eq!(report.kind, FormKind::List);
        assert_eq!(report.head.as_deref(), Some("defun"));
        assert!(report.definition_like);
        assert_eq!(report.child_count, 4);
        assert_eq!(report.list_count, 3);
        assert_eq!(report.source.as_deref(), Some("(defun add (x y) (+ x y))"));
        assert!(report.symbols.iter().any(|symbol| symbol.symbol == "x"));
    }

    #[test]
    fn reports_atom_target_without_head() {
        let report = report_for("(message \"foo\" bar)", "0.2", Dialect::EmacsLisp);

        assert_eq!(report.kind, FormKind::Atom);
        assert_eq!(report.head, None);
        assert!(!report.definition_like);
        assert_eq!(report.atom_count, 1);
        assert_eq!(report.list_count, 0);
        assert_eq!(report.symbols[0].symbol, "bar");
    }
}
