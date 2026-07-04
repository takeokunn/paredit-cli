use crate::application::usecase::rename::selection::span_contains;

use super::WrapFunctionCallSite;

pub(super) fn select_outermost_wrap_call_sites(
    mut candidates: Vec<WrapFunctionCallSite>,
) -> (Vec<WrapFunctionCallSite>, Vec<WrapFunctionCallSite>) {
    candidates.sort_by_key(|site| (site.span.start().get(), std::cmp::Reverse(site.span.len())));

    let mut selected: Vec<WrapFunctionCallSite> = Vec::new();
    let mut skipped_nested = Vec::new();
    for site in candidates {
        if selected
            .iter()
            .any(|selected| span_contains(selected.span, site.span) && selected.span != site.span)
        {
            skipped_nested.push(site);
        } else {
            selected.push(site);
        }
    }
    selected.sort_by_key(|site| site.span.start());
    skipped_nested.sort_by_key(|site| site.span.start());
    (selected, skipped_nested)
}
