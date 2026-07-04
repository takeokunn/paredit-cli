use anyhow::{Context, Result};

use crate::application::usecase::callable_scope::{
    common_lisp_local_callable_form, is_local_callable_bound, local_callable_names,
    LocalCallableForm,
};
use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

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

pub(super) fn do_binding_reference_spans(
    input: &str,
    target: &ExpressionView,
    binding_form: &ExpressionView,
    candidates: &[LetBindingRemovalCandidate],
    candidate: &LetBindingRemovalCandidate,
    name: &SymbolName,
) -> Vec<ByteSpan> {
    let mut reference_spans = Vec::new();
    let sequential_scope = list_head(target).is_some_and(|head| head == "do*");

    if sequential_scope {
        for later in candidates
            .iter()
            .filter(|later| later.index > candidate.index)
        {
            let Some(later_spec) = binding_form.children.get(later.index) else {
                continue;
            };
            if let Some(init_form) = iteration_variable_spec_init_form(later_spec) {
                collect_unshadowed_symbol_references(init_form, name, input, &mut reference_spans);
            }
        }
    }

    for spec in &binding_form.children {
        if let Some(step_form) = do_variable_spec_step_form(spec) {
            collect_unshadowed_symbol_references(step_form, name, input, &mut reference_spans);
        }
    }
    if let Some(end_clause) = target.children.get(2) {
        collect_unshadowed_symbol_references(end_clause, name, input, &mut reference_spans);
    }
    for body in &target.children[3..] {
        collect_unshadowed_symbol_references(body, name, input, &mut reference_spans);
    }

    reference_spans
}

pub(super) fn prog_binding_reference_spans(
    input: &str,
    target: &ExpressionView,
    binding_form: &ExpressionView,
    candidates: &[LetBindingRemovalCandidate],
    candidate: &LetBindingRemovalCandidate,
    name: &SymbolName,
) -> Vec<ByteSpan> {
    let mut reference_spans = Vec::new();
    let sequential_scope = list_head(target).is_some_and(|head| head == "prog*");

    if sequential_scope {
        for later in candidates
            .iter()
            .filter(|later| later.index > candidate.index)
        {
            let Some(later_spec) = binding_form.children.get(later.index) else {
                continue;
            };
            if let Some(init_form) = iteration_variable_spec_init_form(later_spec) {
                collect_unshadowed_symbol_references(init_form, name, input, &mut reference_spans);
            }
        }
    }

    for body in &target.children[2..] {
        collect_unshadowed_symbol_references(body, name, input, &mut reference_spans);
    }

    reference_spans
}

pub(super) fn body_binding_reference_spans(
    input: &str,
    target: &ExpressionView,
    name: &SymbolName,
    body_start_index: usize,
) -> Vec<ByteSpan> {
    let mut reference_spans = Vec::new();
    for body in target.children.iter().skip(body_start_index) {
        collect_unshadowed_symbol_references(body, name, input, &mut reference_spans);
    }
    reference_spans
}

pub(super) fn local_callable_binding_reference_spans(
    dialect: Dialect,
    target: &ExpressionView,
    name: &SymbolName,
) -> Result<Vec<ByteSpan>> {
    let mut reference_spans = Vec::new();
    let Some(head) = list_head(target) else {
        return Ok(reference_spans);
    };
    let Some(form) = common_lisp_local_callable_form(dialect, head) else {
        return Ok(reference_spans);
    };

    collect_local_callable_form_reference_spans(
        target,
        dialect,
        name,
        &[],
        form,
        true,
        &mut reference_spans,
    );

    Ok(reference_spans)
}

#[allow(clippy::too_many_arguments)]
fn collect_local_callable_form_reference_spans(
    view: &ExpressionView,
    dialect: Dialect,
    name: &SymbolName,
    local_callables: &[String],
    form: LocalCallableForm,
    count_this_forms_local_names: bool,
    output: &mut Vec<ByteSpan>,
) {
    let local_names = local_callable_names(view);
    let mut descendant_scope = local_callables.to_vec();
    descendant_scope.extend(local_names.iter().cloned());
    let direct_scope = if count_this_forms_local_names {
        local_callables
    } else {
        descendant_scope.as_slice()
    };

    if let Some(bindings) = view.children.get(1) {
        let binding_body_scope = match form {
            LocalCallableForm::Labels => direct_scope,
            LocalCallableForm::Flet
            | LocalCallableForm::Macrolet
            | LocalCallableForm::CompilerMacrolet => descendant_scope.as_slice(),
        };
        for binding in &bindings.children {
            for child in binding.children.iter().skip(2) {
                collect_local_callable_reference_spans_from_view(
                    child,
                    dialect,
                    name,
                    binding_body_scope,
                    output,
                );
            }
        }
    }

    for child in view.children.iter().skip(2) {
        collect_local_callable_reference_spans_from_view(
            child,
            dialect,
            name,
            direct_scope,
            output,
        );
    }
}

fn collect_local_callable_reference_spans_from_view(
    view: &ExpressionView,
    dialect: Dialect,
    name: &SymbolName,
    local_callables: &[String],
    output: &mut Vec<ByteSpan>,
) {
    let mut first_body_child_index = 0;

    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        if let Some(head) = list_head(view) {
            if let Some(form) = common_lisp_local_callable_form(dialect, head) {
                collect_local_callable_form_reference_spans(
                    view,
                    dialect,
                    name,
                    local_callables,
                    form,
                    false,
                    output,
                );
                return;
            }

            if head == name.as_str() && !is_local_callable_bound(local_callables, head) {
                if let Some(head_view) = view.children.first() {
                    output.push(head_view.span);
                }
            }

            if let Some(category) = classify_definition_head(dialect, head) {
                first_body_child_index = definition_body_start_index(category);
            }
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        if index < first_body_child_index {
            continue;
        }
        collect_local_callable_reference_spans_from_view(
            child,
            dialect,
            name,
            local_callables,
            output,
        );
    }
}

fn definition_body_start_index(category: crate::domain::definition::DefinitionCategory) -> usize {
    if category.is_callable() {
        3
    } else {
        2
    }
}

fn iteration_variable_spec_init_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    if spec.kind == ExpressionKind::List {
        spec.children.get(1)
    } else {
        None
    }
}

fn do_variable_spec_step_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    if spec.kind == ExpressionKind::List {
        spec.children.get(2)
    } else {
        None
    }
}
