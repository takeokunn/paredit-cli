use crate::domain::sexpr::{ByteOffset, ByteSpan, ExpressionView};

use super::syntax::binding_form_binds_name;

#[derive(Debug, Default)]
pub(super) struct EquivalentExpressionSpans {
    pub(super) replacement_spans: Vec<ByteSpan>,
    pub(super) skipped_shadowed_spans: Vec<ByteSpan>,
}

pub(super) fn collect_equivalent_expression_spans(
    view: &ExpressionView,
    target: &ExpressionView,
    binding_name: &str,
    shadowed_by_binding: bool,
    output: &mut EquivalentExpressionSpans,
) {
    if expressions_equivalent(view, target) {
        if shadowed_by_binding {
            output.skipped_shadowed_spans.push(view.span);
        } else {
            output.replacement_spans.push(view.span);
        }
        return;
    }

    let child_shadowed = shadowed_by_binding || binding_form_binds_name(view, binding_name);
    for child in &view.children {
        collect_equivalent_expression_spans(child, target, binding_name, child_shadowed, output);
    }
}

pub(super) fn is_span_shadowed_by_binding(
    view: &ExpressionView,
    target_span: ByteSpan,
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    if view.span == target_span {
        return shadowed_by_binding;
    }

    let child_shadowed = shadowed_by_binding || binding_form_binds_name(view, binding_name);
    view.children
        .iter()
        .any(|child| is_span_shadowed_by_binding(child, target_span, binding_name, child_shadowed))
}

pub(super) fn rebase_spans(mut spans: Vec<ByteSpan>, base: ByteOffset) -> Vec<ByteSpan> {
    spans.sort_by_key(|span| span.start());
    spans.dedup();
    spans
        .into_iter()
        .map(|span| {
            ByteSpan::new(
                ByteOffset::new(span.start().get() + base.get()),
                ByteOffset::new(span.end().get() + base.get()),
            )
        })
        .collect()
}

fn expressions_equivalent(left: &ExpressionView, right: &ExpressionView) -> bool {
    left.kind == right.kind
        && left.delimiter == right.delimiter
        && left.text == right.text
        && left.children.len() == right.children.len()
        && left
            .children
            .iter()
            .zip(&right.children)
            .all(|(left, right)| expressions_equivalent(left, right))
}
