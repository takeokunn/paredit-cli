use anyhow::{Context, Result};

use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionView, SymbolName};

use super::candidates::LetBindingRemovalCandidate;
use super::syntax::{list_head, view_at_span};

pub(super) fn let_binding_reference_spans(
    input: &str,
    target: &ExpressionView,
    binding_form: &ExpressionView,
    candidates: &[LetBindingRemovalCandidate],
    candidate: &LetBindingRemovalCandidate,
    name: &SymbolName,
) -> Result<Vec<ByteSpan>> {
    let mut reference_spans = Vec::new();
    let sequential_scope = list_head(target).is_some_and(|head| head == "let*")
        || binding_form.delimiter == Some(Delimiter::Bracket);
    if sequential_scope {
        for later in candidates
            .iter()
            .filter(|later| later.index > candidate.index)
        {
            let later_value = view_at_span(binding_form, later.value_span)
                .context("failed to resolve later binding value")?;
            collect_unshadowed_symbol_references(later_value, name, input, &mut reference_spans);
        }
    }
    for body in &target.children[2..] {
        collect_unshadowed_symbol_references(body, name, input, &mut reference_spans);
    }
    Ok(reference_spans)
}
