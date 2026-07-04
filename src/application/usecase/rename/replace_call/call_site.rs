use crate::application::usecase::rename::selection::list_head;
use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, SymbolName};

use super::ReplaceFunctionCallSite;

pub(super) fn replace_call_site_from_view(
    view: &ExpressionView,
    dialect: Dialect,
    input: &str,
    path: String,
    from: &SymbolName,
    to: &SymbolName,
) -> Option<ReplaceFunctionCallSite> {
    let head = list_head(view)?;
    if head != from.as_str() || classify_definition_head(dialect, head).is_some() {
        return None;
    }
    let head_span = view.children.first().map(|child| child.span)?;

    Some(ReplaceFunctionCallSite {
        path,
        head_span,
        span: view.span,
        replacement: to.as_str().to_owned(),
        text: view.span.slice(input).to_owned(),
    })
}
