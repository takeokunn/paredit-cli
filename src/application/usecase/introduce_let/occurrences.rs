use crate::domain::sexpr::{ByteOffset, ByteSpan, ChildIndex, ExpressionView};

use super::syntax::{
    binding_pair_binds_name, child_shadowed_by_binding, let_star_bindings_child_index,
    local_callable_binding_child_shadowed, local_callable_bindings_child_index,
};

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

    for (index, child) in view.children.iter().enumerate() {
        if let_star_bindings_child_index(view) == Some(index) {
            collect_let_star_binding_spans(
                child,
                target,
                binding_name,
                shadowed_by_binding,
                output,
            );
            continue;
        }

        if local_callable_bindings_child_index(view) == Some(index) {
            collect_local_callable_binding_spans(
                child,
                target,
                binding_name,
                shadowed_by_binding,
                output,
            );
            continue;
        }

        let child_shadowed =
            shadowed_by_binding || child_shadowed_by_binding(view, binding_name, index);
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

    view.children.iter().enumerate().any(|(index, child)| {
        if let_star_bindings_child_index(view) == Some(index) {
            return is_span_shadowed_by_let_star_bindings(
                child,
                target_span,
                binding_name,
                shadowed_by_binding,
            );
        }

        if local_callable_bindings_child_index(view) == Some(index) {
            return is_span_shadowed_by_local_callable_binding(
                child,
                target_span,
                binding_name,
                shadowed_by_binding,
            );
        }

        let child_shadowed =
            shadowed_by_binding || child_shadowed_by_binding(view, binding_name, index);
        is_span_shadowed_by_binding(child, target_span, binding_name, child_shadowed)
    })
}

pub(super) fn is_path_shadowed_by_binding(
    view: &ExpressionView,
    target_path: &[ChildIndex],
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    let Some((index, rest)) = target_path.split_first() else {
        return shadowed_by_binding;
    };
    let index = index.get();
    let Some(child) = view.children.get(index) else {
        return false;
    };

    if let_star_bindings_child_index(view) == Some(index) {
        return is_path_shadowed_by_let_star_bindings(
            child,
            rest,
            binding_name,
            shadowed_by_binding,
        );
    }

    if local_callable_bindings_child_index(view) == Some(index) {
        return is_path_shadowed_by_local_callable_binding(
            child,
            rest,
            binding_name,
            shadowed_by_binding,
        );
    }

    let child_shadowed =
        shadowed_by_binding || child_shadowed_by_binding(view, binding_name, index);
    if rest.is_empty() {
        child_shadowed
    } else {
        is_path_shadowed_by_binding(child, rest, binding_name, child_shadowed)
    }
}

fn collect_let_star_binding_spans(
    binding_form: &ExpressionView,
    target: &ExpressionView,
    binding_name: &str,
    shadowed_by_binding: bool,
    output: &mut EquivalentExpressionSpans,
) {
    if expressions_equivalent(binding_form, target) {
        if shadowed_by_binding {
            output.skipped_shadowed_spans.push(binding_form.span);
        } else {
            output.replacement_spans.push(binding_form.span);
        }
        return;
    }

    let mut sequential_shadowed = shadowed_by_binding;
    for binding in &binding_form.children {
        collect_let_star_binding_spec_spans(
            binding,
            target,
            binding_name,
            sequential_shadowed,
            output,
        );
        if binding_pair_binds_name(binding, binding_name) {
            sequential_shadowed = true;
        }
    }
}

fn collect_let_star_binding_spec_spans(
    binding: &ExpressionView,
    target: &ExpressionView,
    binding_name: &str,
    shadowed_by_binding: bool,
    output: &mut EquivalentExpressionSpans,
) {
    if expressions_equivalent(binding, target) {
        if shadowed_by_binding {
            output.skipped_shadowed_spans.push(binding.span);
        } else {
            output.replacement_spans.push(binding.span);
        }
        return;
    }

    for child in &binding.children {
        collect_equivalent_expression_spans(
            child,
            target,
            binding_name,
            shadowed_by_binding,
            output,
        );
    }
}

fn is_span_shadowed_by_let_star_bindings(
    binding_form: &ExpressionView,
    target_span: ByteSpan,
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    if binding_form.span == target_span {
        return shadowed_by_binding;
    }

    let mut sequential_shadowed = shadowed_by_binding;
    for binding in &binding_form.children {
        if span_contains_span(binding.span, target_span) {
            return is_span_shadowed_by_binding(
                binding,
                target_span,
                binding_name,
                sequential_shadowed,
            );
        }
        if binding_pair_binds_name(binding, binding_name) {
            sequential_shadowed = true;
        }
    }

    false
}

fn span_contains_span(container: ByteSpan, contained: ByteSpan) -> bool {
    container.start().get() <= contained.start().get()
        && contained.end().get() <= container.end().get()
}

fn is_path_shadowed_by_let_star_bindings(
    binding_form: &ExpressionView,
    target_path: &[ChildIndex],
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    let Some((index, rest)) = target_path.split_first() else {
        return shadowed_by_binding;
    };
    let index = index.get();

    let mut sequential_shadowed = shadowed_by_binding;
    for (binding_index, binding) in binding_form.children.iter().enumerate() {
        if binding_index == index {
            return if rest.is_empty() {
                sequential_shadowed
            } else {
                is_path_shadowed_by_binding(binding, rest, binding_name, sequential_shadowed)
            };
        }
        if binding_pair_binds_name(binding, binding_name) {
            sequential_shadowed = true;
        }
    }

    false
}

fn collect_local_callable_binding_spans(
    binding_form: &ExpressionView,
    target: &ExpressionView,
    binding_name: &str,
    shadowed_by_binding: bool,
    output: &mut EquivalentExpressionSpans,
) {
    if expressions_equivalent(binding_form, target) {
        if shadowed_by_binding {
            output.skipped_shadowed_spans.push(binding_form.span);
        } else {
            output.replacement_spans.push(binding_form.span);
        }
        return;
    }

    for binding in &binding_form.children {
        collect_local_callable_binding_spec_spans(
            binding,
            target,
            binding_name,
            shadowed_by_binding,
            output,
        );
    }
}

fn collect_local_callable_binding_spec_spans(
    binding: &ExpressionView,
    target: &ExpressionView,
    binding_name: &str,
    shadowed_by_binding: bool,
    output: &mut EquivalentExpressionSpans,
) {
    if expressions_equivalent(binding, target) {
        if shadowed_by_binding {
            output.skipped_shadowed_spans.push(binding.span);
        } else {
            output.replacement_spans.push(binding.span);
        }
        return;
    }

    for (index, child) in binding.children.iter().enumerate() {
        let child_shadowed = shadowed_by_binding
            || local_callable_binding_child_shadowed(binding, binding_name, index);
        collect_equivalent_expression_spans(child, target, binding_name, child_shadowed, output);
    }
}

fn is_span_shadowed_by_local_callable_binding(
    binding_form: &ExpressionView,
    target_span: ByteSpan,
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    if binding_form.span == target_span {
        return shadowed_by_binding;
    }

    binding_form.children.iter().any(|binding| {
        is_span_shadowed_by_local_callable_binding_spec(
            binding,
            target_span,
            binding_name,
            shadowed_by_binding,
        )
    })
}

fn is_path_shadowed_by_local_callable_binding(
    binding_form: &ExpressionView,
    target_path: &[ChildIndex],
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    let Some((index, rest)) = target_path.split_first() else {
        return shadowed_by_binding;
    };
    let index = index.get();
    let Some(binding) = binding_form.children.get(index) else {
        return false;
    };

    if rest.is_empty() {
        shadowed_by_binding
    } else {
        is_path_shadowed_by_local_callable_binding_spec(
            binding,
            rest,
            binding_name,
            shadowed_by_binding,
        )
    }
}

fn is_span_shadowed_by_local_callable_binding_spec(
    binding: &ExpressionView,
    target_span: ByteSpan,
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    if binding.span == target_span {
        return shadowed_by_binding;
    }

    binding.children.iter().enumerate().any(|(index, child)| {
        let child_shadowed = shadowed_by_binding
            || local_callable_binding_child_shadowed(binding, binding_name, index);
        is_span_shadowed_by_binding(child, target_span, binding_name, child_shadowed)
    })
}

fn is_path_shadowed_by_local_callable_binding_spec(
    binding: &ExpressionView,
    target_path: &[ChildIndex],
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    let Some((index, rest)) = target_path.split_first() else {
        return shadowed_by_binding;
    };
    let index = index.get();
    let Some(child) = binding.children.get(index) else {
        return false;
    };

    let child_shadowed =
        shadowed_by_binding || local_callable_binding_child_shadowed(binding, binding_name, index);
    if rest.is_empty() {
        child_shadowed
    } else {
        is_path_shadowed_by_binding(child, rest, binding_name, child_shadowed)
    }
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
