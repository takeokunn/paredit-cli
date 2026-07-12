use anyhow::Result;

use crate::application::usecase::callable_scope::{
    common_lisp_local_callable_form, is_local_callable_bound, local_callable_body_scope,
};
use crate::domain::common_lisp::{
    CommonLispLocalCallableForm, LocalCallableName, common_lisp_operator_head_eq,
    common_lisp_symbol_reference_eq, local_callable_definition_reference_scope,
};
use crate::domain::definition::definition_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, ReaderPrefix, SymbolName,
};

use super::super::syntax::list_head;

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

fn collect_local_callable_form_reference_spans(
    view: &ExpressionView,
    dialect: Dialect,
    name: &SymbolName,
    local_callables: &[LocalCallableName],
    form: CommonLispLocalCallableForm,
    count_this_forms_local_names: bool,
    output: &mut Vec<ByteSpan>,
) {
    let descendant_scope = local_callable_body_scope(local_callables, view);
    let direct_scope = if count_this_forms_local_names {
        local_callables
    } else {
        descendant_scope.as_slice()
    };

    if let Some(bindings) = view.children.get(1) {
        let binding_body_scope = local_callable_definition_reference_scope(
            form,
            direct_scope,
            descendant_scope.as_slice(),
        );
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
    local_callables: &[LocalCallableName],
    output: &mut Vec<ByteSpan>,
) {
    if view.kind == ExpressionKind::Atom
        && view.reader_prefixes.contains(&ReaderPrefix::Function)
        && view
            .text
            .as_deref()
            .is_some_and(|symbol| common_lisp_symbol_reference_eq(symbol, name.as_str()))
        && !view
            .text
            .as_deref()
            .is_some_and(|symbol| is_local_callable_bound(local_callables, symbol))
    {
        output.push(view.span);
    }

    let mut definition_body_range = None;

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

            if common_lisp_symbol_reference_eq(head, name.as_str())
                && !is_local_callable_bound(local_callables, head)
            {
                if let Some(head_view) = view.children.first() {
                    output.push(head_view.span);
                }
            }

            if common_lisp_operator_head_eq(head, "function") {
                if let Some(designator) = view.children.get(1) {
                    if designator.kind == ExpressionKind::Atom
                        && designator.text.as_deref().is_some_and(|symbol| {
                            common_lisp_symbol_reference_eq(symbol, name.as_str())
                        })
                        && !designator
                            .text
                            .as_deref()
                            .is_some_and(|symbol| is_local_callable_bound(local_callables, symbol))
                    {
                        output.push(designator.span);
                    }
                }
            }

            if let Some(shape) = definition_shape(dialect, view, head) {
                definition_body_range = Some(shape.body_range());
            }
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        if let Some(range) = definition_body_range {
            if !range.contains_child(index) {
                continue;
            }
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
