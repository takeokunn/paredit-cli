use anyhow::{Context, Result};

use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{Delimiter, ExpressionView, SymbolName};

use super::super::syntax::{atom_text, list_head, view_at_span};
use super::bindings::LetBindingCandidate;

pub(super) fn let_binding_reference_count(
    input: &str,
    view: &ExpressionView,
    binding_form: &ExpressionView,
    candidates: &[LetBindingCandidate],
    candidate: &LetBindingCandidate,
    symbol: &SymbolName,
) -> Result<usize> {
    let mut reference_spans = Vec::new();
    let sequential_scope = list_head(view).is_some_and(|head| head == "let*")
        || binding_form.delimiter == Some(Delimiter::Bracket);
    if sequential_scope {
        for later in candidates
            .iter()
            .filter(|later| later.index > candidate.index)
        {
            let later_value = view_at_span(binding_form, later.value_span)
                .context("failed to resolve later binding value")?;
            collect_unshadowed_symbol_references(later_value, symbol, input, &mut reference_spans);
        }
    }
    for body in &view.children[2..] {
        collect_unshadowed_symbol_references(body, symbol, input, &mut reference_spans);
    }
    Ok(reference_spans.len())
}

pub(super) fn fallback_reference_count(view: &ExpressionView, symbol: &str) -> usize {
    usize::from(atom_text(view).is_some_and(|text| text == symbol))
        + view
            .children
            .iter()
            .map(|child| fallback_reference_count(child, symbol))
            .sum::<usize>()
}
