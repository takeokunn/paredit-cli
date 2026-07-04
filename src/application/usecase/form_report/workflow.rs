use anyhow::Result;

use crate::application::usecase::form_report::stats::FormStats;
use crate::application::usecase::form_report::types::{FormKind, FormReport, FormReportRequest};
use crate::domain::sexpr::{ExpressionKind, ExpressionView};

pub fn build_form_report(request: FormReportRequest<'_>) -> Result<FormReport> {
    let stats = FormStats::collect(&request.target);

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
        symbols: stats.into_symbols(),
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
