use crate::domain::common_lisp::{CommonLispVariableBindingForm, CommonLispVariableSpecForm};
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView};

use super::{BindingReferenceContext, body::body_binding_reference_spans};

pub(super) fn variable_spec_binding_reference_spans(
    context: &BindingReferenceContext<'_>,
    spec_form: CommonLispVariableSpecForm,
    binding_form_kind: CommonLispVariableBindingForm,
) -> Vec<ByteSpan> {
    let mut reference_spans = Vec::new();

    if binding_form_kind.is_sequential() {
        for later in context
            .candidates
            .iter()
            .filter(|later| later.index > context.candidate.index)
        {
            let Some(later_spec) = context.binding_form.children.get(later.index) else {
                continue;
            };
            if let Some(init_form) = iteration_variable_spec_init_form(later_spec) {
                collect_unshadowed_symbol_references(
                    context.dialect,
                    init_form,
                    context.name,
                    context.input,
                    &mut reference_spans,
                );
            }
        }
    }

    if spec_form.has_step_forms() {
        for spec in &context.binding_form.children {
            if let Some(step_form) = variable_spec_step_form(spec) {
                collect_unshadowed_symbol_references(
                    context.dialect,
                    step_form,
                    context.name,
                    context.input,
                    &mut reference_spans,
                );
            }
        }
    }

    if let Some(end_clause_index) = spec_form.end_clause_index() {
        if let Some(end_clause) = context.target.children.get(end_clause_index) {
            collect_unshadowed_symbol_references(
                context.dialect,
                end_clause,
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
        spec_form.body_start_index(),
    ));

    reference_spans
}

fn iteration_variable_spec_init_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    (spec.kind == ExpressionKind::List)
        .then(|| spec.children.get(1))
        .flatten()
}

fn variable_spec_step_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    (spec.kind == ExpressionKind::List)
        .then(|| spec.children.get(2))
        .flatten()
}
