use crate::application::usecase::rename::selection::list_head;
use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, SymbolName};

use super::UnwrapFunctionCallSite;

pub(super) enum UnwrapCandidate {
    Selected(UnwrapFunctionCallSite),
    NonUnaryWrapper(UnwrapFunctionCallSite),
    NotMatched,
}

pub(super) fn unwrap_call_site_from_view(
    view: &ExpressionView,
    dialect: Dialect,
    input: &str,
    path: String,
    function: &SymbolName,
    wrapper: &SymbolName,
) -> UnwrapCandidate {
    let Some(head) = list_head(view) else {
        return UnwrapCandidate::NotMatched;
    };
    if head != wrapper.as_str() || classify_definition_head(dialect, head).is_some() {
        return UnwrapCandidate::NotMatched;
    }

    let matching_inner_call = view
        .children
        .iter()
        .skip(1)
        .find(|child| list_head(child).is_some_and(|head| head == function.as_str()));
    let Some(inner_call) = matching_inner_call else {
        return UnwrapCandidate::NotMatched;
    };

    let site = UnwrapFunctionCallSite {
        path,
        span: view.span,
        replacement: inner_call.span.slice(input).to_owned(),
        text: view.span.slice(input).to_owned(),
    };
    if view.children.len() == 2 {
        UnwrapCandidate::Selected(site)
    } else {
        UnwrapCandidate::NonUnaryWrapper(site)
    }
}
