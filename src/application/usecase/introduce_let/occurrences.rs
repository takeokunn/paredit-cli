use crate::domain::sexpr::{ByteOffset, ByteSpan, ChildIndex, ExpressionView};

use super::syntax::{
    binding_pair_binds_name, child_shadowed_by_binding, iteration_binding_child_shadowed,
    iteration_bindings_child_index, let_star_bindings_child_index,
    local_callable_binding_child_shadowed, local_callable_bindings_child_index,
    variable_binding_form_has_step_forms, variable_binding_form_is_sequential,
    variable_bindings_child_index, variable_spec_binds_name,
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

        if iteration_bindings_child_index(view) == Some(index) {
            collect_iteration_binding_spans(
                child,
                target,
                binding_name,
                shadowed_by_binding,
                output,
            );
            continue;
        }

        if variable_bindings_child_index(view) == Some(index) {
            collect_variable_binding_spans(
                child,
                target,
                binding_name,
                shadowed_by_binding,
                variable_binding_form_is_sequential(view),
                variable_binding_form_has_step_forms(view),
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

        if iteration_bindings_child_index(view) == Some(index) {
            return is_span_shadowed_by_iteration_bindings(
                child,
                target_span,
                binding_name,
                shadowed_by_binding,
            );
        }

        if variable_bindings_child_index(view) == Some(index) {
            return is_span_shadowed_by_variable_bindings(
                child,
                target_span,
                binding_name,
                shadowed_by_binding,
                variable_binding_form_is_sequential(view),
                variable_binding_form_has_step_forms(view),
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

    if iteration_bindings_child_index(view) == Some(index) {
        return is_path_shadowed_by_iteration_bindings(
            child,
            rest,
            binding_name,
            shadowed_by_binding,
        );
    }

    if variable_bindings_child_index(view) == Some(index) {
        return is_path_shadowed_by_variable_bindings(
            child,
            rest,
            binding_name,
            shadowed_by_binding,
            variable_binding_form_is_sequential(view),
            variable_binding_form_has_step_forms(view),
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

fn collect_iteration_binding_spans(
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

    for (index, child) in binding_form.children.iter().enumerate() {
        let child_shadowed = shadowed_by_binding
            || iteration_binding_child_shadowed(binding_form, binding_name, index);
        collect_equivalent_expression_spans(child, target, binding_name, child_shadowed, output);
    }
}

fn is_span_shadowed_by_iteration_bindings(
    binding_form: &ExpressionView,
    target_span: ByteSpan,
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    if binding_form.span == target_span {
        return shadowed_by_binding;
    }

    binding_form
        .children
        .iter()
        .enumerate()
        .any(|(index, child)| {
            let child_shadowed = shadowed_by_binding
                || iteration_binding_child_shadowed(binding_form, binding_name, index);
            is_span_shadowed_by_binding(child, target_span, binding_name, child_shadowed)
        })
}

fn is_path_shadowed_by_iteration_bindings(
    binding_form: &ExpressionView,
    target_path: &[ChildIndex],
    binding_name: &str,
    shadowed_by_binding: bool,
) -> bool {
    let Some((index, rest)) = target_path.split_first() else {
        return shadowed_by_binding;
    };
    let index = index.get();
    let Some(child) = binding_form.children.get(index) else {
        return false;
    };

    let child_shadowed =
        shadowed_by_binding || iteration_binding_child_shadowed(binding_form, binding_name, index);
    if rest.is_empty() {
        child_shadowed
    } else {
        is_path_shadowed_by_binding(child, rest, binding_name, child_shadowed)
    }
}

fn collect_variable_binding_spans(
    binding_form: &ExpressionView,
    target: &ExpressionView,
    binding_name: &str,
    shadowed_by_binding: bool,
    sequential: bool,
    has_step_forms: bool,
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

    let all_bindings_shadowed = shadowed_by_binding
        || binding_form
            .children
            .iter()
            .any(|binding| variable_spec_binds_name(binding, binding_name));
    let mut sequential_shadowed = shadowed_by_binding;

    for binding in &binding_form.children {
        collect_variable_binding_spec_spans(
            binding,
            target,
            binding_name,
            sequential_shadowed,
            all_bindings_shadowed,
            has_step_forms,
            output,
        );
        if sequential && variable_spec_binds_name(binding, binding_name) {
            sequential_shadowed = true;
        }
    }
}

fn collect_variable_binding_spec_spans(
    binding: &ExpressionView,
    target: &ExpressionView,
    binding_name: &str,
    init_shadowed: bool,
    step_shadowed: bool,
    has_step_forms: bool,
    output: &mut EquivalentExpressionSpans,
) {
    if expressions_equivalent(binding, target) {
        if init_shadowed {
            output.skipped_shadowed_spans.push(binding.span);
        } else {
            output.replacement_spans.push(binding.span);
        }
        return;
    }

    for (index, child) in binding.children.iter().enumerate() {
        let child_shadowed = variable_binding_spec_child_shadowed(
            index,
            init_shadowed,
            step_shadowed,
            has_step_forms,
        );
        collect_equivalent_expression_spans(child, target, binding_name, child_shadowed, output);
    }
}

fn is_span_shadowed_by_variable_bindings(
    binding_form: &ExpressionView,
    target_span: ByteSpan,
    binding_name: &str,
    shadowed_by_binding: bool,
    sequential: bool,
    has_step_forms: bool,
) -> bool {
    if binding_form.span == target_span {
        return shadowed_by_binding;
    }

    let all_bindings_shadowed = shadowed_by_binding
        || binding_form
            .children
            .iter()
            .any(|binding| variable_spec_binds_name(binding, binding_name));
    let mut sequential_shadowed = shadowed_by_binding;

    for binding in &binding_form.children {
        if span_contains_span(binding.span, target_span) {
            return is_span_shadowed_by_variable_binding_spec(
                binding,
                target_span,
                binding_name,
                sequential_shadowed,
                all_bindings_shadowed,
                has_step_forms,
            );
        }
        if sequential && variable_spec_binds_name(binding, binding_name) {
            sequential_shadowed = true;
        }
    }

    false
}

fn is_span_shadowed_by_variable_binding_spec(
    binding: &ExpressionView,
    target_span: ByteSpan,
    binding_name: &str,
    init_shadowed: bool,
    step_shadowed: bool,
    has_step_forms: bool,
) -> bool {
    if binding.span == target_span {
        return init_shadowed;
    }

    binding.children.iter().enumerate().any(|(index, child)| {
        let child_shadowed = variable_binding_spec_child_shadowed(
            index,
            init_shadowed,
            step_shadowed,
            has_step_forms,
        );
        is_span_shadowed_by_binding(child, target_span, binding_name, child_shadowed)
    })
}

fn is_path_shadowed_by_variable_bindings(
    binding_form: &ExpressionView,
    target_path: &[ChildIndex],
    binding_name: &str,
    shadowed_by_binding: bool,
    sequential: bool,
    has_step_forms: bool,
) -> bool {
    let Some((index, rest)) = target_path.split_first() else {
        return shadowed_by_binding;
    };
    let index = index.get();

    let all_bindings_shadowed = shadowed_by_binding
        || binding_form
            .children
            .iter()
            .any(|binding| variable_spec_binds_name(binding, binding_name));
    let mut sequential_shadowed = shadowed_by_binding;

    for (binding_index, binding) in binding_form.children.iter().enumerate() {
        if binding_index == index {
            return if rest.is_empty() {
                sequential_shadowed
            } else {
                is_path_shadowed_by_variable_binding_spec(
                    binding,
                    rest,
                    binding_name,
                    sequential_shadowed,
                    all_bindings_shadowed,
                    has_step_forms,
                )
            };
        }
        if sequential && variable_spec_binds_name(binding, binding_name) {
            sequential_shadowed = true;
        }
    }

    false
}

fn is_path_shadowed_by_variable_binding_spec(
    binding: &ExpressionView,
    target_path: &[ChildIndex],
    binding_name: &str,
    init_shadowed: bool,
    step_shadowed: bool,
    has_step_forms: bool,
) -> bool {
    let Some((index, rest)) = target_path.split_first() else {
        return init_shadowed;
    };
    let index = index.get();
    let Some(child) = binding.children.get(index) else {
        return false;
    };

    let child_shadowed =
        variable_binding_spec_child_shadowed(index, init_shadowed, step_shadowed, has_step_forms);
    if rest.is_empty() {
        child_shadowed
    } else {
        is_path_shadowed_by_binding(child, rest, binding_name, child_shadowed)
    }
}

fn variable_binding_spec_child_shadowed(
    child_index: usize,
    init_shadowed: bool,
    step_shadowed: bool,
    has_step_forms: bool,
) -> bool {
    match child_index {
        0 | 1 => init_shadowed,
        _ if has_step_forms => step_shadowed,
        _ => init_shadowed,
    }
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
