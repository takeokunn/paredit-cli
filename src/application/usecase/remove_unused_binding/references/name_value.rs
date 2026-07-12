use anyhow::{Context, Result};

use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::ByteSpan;

use super::super::syntax::view_at_span;
use super::{BindingReferenceContext, body::body_binding_reference_spans};

pub(super) fn name_value_binding_reference_spans(
    context: &BindingReferenceContext<'_>,
    sequential_scope: bool,
) -> Result<Vec<ByteSpan>> {
    let mut reference_spans = Vec::new();
    if sequential_scope {
        for later in context
            .candidates
            .iter()
            .filter(|later| later.index > context.candidate.index)
        {
            let later_value = view_at_span(context.binding_form, later.value_span)
                .context("failed to resolve later binding value")?;
            collect_unshadowed_symbol_references(
                context.dialect,
                later_value,
                context.name,
                context.input,
                &mut reference_spans,
            );
        }
    }
    reference_spans.extend(body_binding_reference_spans(
        context.dialect,
        context.input,
        context.target,
        context.name,
        2,
    ));
    Ok(reference_spans)
}
