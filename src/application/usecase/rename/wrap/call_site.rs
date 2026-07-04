use crate::domain::sexpr::{ExpressionView, SymbolName};

use super::WrapFunctionCallSite;
use crate::application::usecase::rename::selection::list_head;

pub(super) fn wrap_call_site_from_view(
    view: &ExpressionView,
    input: &str,
    path: String,
    function: &SymbolName,
    wrapper: &SymbolName,
) -> Option<WrapFunctionCallSite> {
    let head = list_head(view)?;
    if head != function.as_str() {
        return None;
    }
    let text = view.span.slice(input).to_owned();
    Some(WrapFunctionCallSite {
        path,
        span: view.span,
        replacement: format!("({} {})", wrapper.as_str(), text),
        text,
    })
}
