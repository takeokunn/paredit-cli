mod destructure;
mod forms;
mod rewrite;
mod scope;
mod types;

use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::selection::{atom_text, list_head};
use forms::{binding_groups, parameter_name_spans};
use scope::collect_symbol_atom_spans_unshadowed;
pub(super) use types::BindingRenameParts;

pub(super) fn binding_rename_parts(
    dialect: Dialect,
    view: &ExpressionView,
    from: &SymbolName,
    input: &str,
) -> Result<BindingRenameParts> {
    let form = list_head(view)
        .context("selected form is not a supported binding form")?
        .to_owned();

    match form.as_str() {
        "let" | "let*" | "symbol-macrolet" => {
            let_binding_rename_parts(dialect, view, from, form, input)
        }
        "destructuring-bind" | "multiple-value-bind" => {
            value_binding_rename_parts(view, from, form, 1, 3, input)
        }
        "lambda" | "fn" => parameter_binding_rename_parts(view, from, form, 1, 2, input),
        "defun" | "defmacro" | "define-setf-expander" | "define-compiler-macro" => {
            parameter_binding_rename_parts(view, from, form, 2, 3, input)
        }
        "handler-case" | "restart-case" => clause_binding_rename_parts(view, from, form, input),
        "dolist" | "dotimes" => iteration_binding_rename_parts(view, from, form, input),
        "with-slots" | "with-accessors" => slot_binding_rename_parts(view, from, form, input),
        _ => anyhow::bail!("selected form is not a supported binding form"),
    }
}

fn let_binding_rename_parts(
    dialect: Dialect,
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let binding_form = view
        .children
        .get(1)
        .context("selected let form must contain bindings")?;
    let bindings = binding_groups(dialect, binding_form, input)?;
    let (target_index, target) = bindings
        .iter()
        .enumerate()
        .find_map(|(index, binding)| {
            binding
                .names
                .iter()
                .find(|name| name.name == from.as_str())
                .map(|name| (index, name))
        })
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected let"))?;

    let sequential_scope = form == "let*" || binding_form.delimiter == Some(Delimiter::Bracket);
    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    if sequential_scope {
        for later in bindings.iter().skip(target_index + 1) {
            collect_symbol_atom_spans_unshadowed(
                &later.value,
                from,
                &mut reference_spans,
                &mut shadowed_scope_count,
                input,
            );
        }
    }

    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(
            body,
            from,
            &mut reference_spans,
            &mut shadowed_scope_count,
            input,
        );
    }
    reference_spans.sort_by_key(|span| span.start());

    Ok(BindingRenameParts {
        form,
        form_span: view.span,
        binding_span: target.name_span,
        binding_edit: target.binding_edit.clone(),
        reference_spans,
        shadowed_scope_count,
    })
}

fn parameter_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    parameter_index: usize,
    body_start_index: usize,
    input: &str,
) -> Result<BindingRenameParts> {
    let parameter_form = view
        .children
        .get(parameter_index)
        .with_context(|| format!("selected {form} form must contain parameters"))?;
    let parameters = parameter_name_spans(parameter_form, input)?;
    let target = parameters
        .iter()
        .find(|parameter| parameter.name == from.as_str())
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    for body in &view.children[body_start_index..] {
        collect_symbol_atom_spans_unshadowed(
            body,
            from,
            &mut reference_spans,
            &mut shadowed_scope_count,
            input,
        );
    }
    reference_spans.sort_by_key(|span| span.start());

    Ok(BindingRenameParts {
        form,
        form_span: view.span,
        binding_span: target.name_span,
        binding_edit: target.binding_edit.clone(),
        reference_spans,
        shadowed_scope_count,
    })
}

fn value_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    binding_index: usize,
    body_start_index: usize,
    input: &str,
) -> Result<BindingRenameParts> {
    let binding_form = view
        .children
        .get(binding_index)
        .with_context(|| format!("selected {form} form must contain bindings"))?;
    let bindings = parameter_name_spans(binding_form, input)?;
    let target = bindings
        .iter()
        .find(|binding| binding.name == from.as_str())
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    for body in &view.children[body_start_index..] {
        collect_symbol_atom_spans_unshadowed(
            body,
            from,
            &mut reference_spans,
            &mut shadowed_scope_count,
            input,
        );
    }
    reference_spans.sort_by_key(|span| span.start());

    Ok(BindingRenameParts {
        form,
        form_span: view.span,
        binding_span: target.name_span,
        binding_edit: target.binding_edit.clone(),
        reference_spans,
        shadowed_scope_count,
    })
}

fn clause_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let mut target = None;
    let mut duplicate_count = 0usize;

    for clause in &view.children[2..] {
        if clause.kind != ExpressionKind::List || clause.delimiter != Some(Delimiter::Paren) {
            continue;
        }

        let Some(parameter_form) = clause.children.get(1) else {
            continue;
        };
        let parameters = parameter_name_spans(parameter_form, input)?;
        let Some(parameter) = parameters
            .iter()
            .find(|parameter| parameter.name == from.as_str())
        else {
            continue;
        };

        duplicate_count += 1;
        target = Some((clause, parameter.clone()));
    }

    if duplicate_count > 1 {
        anyhow::bail!(
            "binding '{from}' was found in multiple selected {form} clauses; select an unambiguous binding form"
        );
    }

    let (target_clause, target_parameter) = target
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    for body in &target_clause.children[2..] {
        collect_symbol_atom_spans_unshadowed(
            body,
            from,
            &mut reference_spans,
            &mut shadowed_scope_count,
            input,
        );
    }
    reference_spans.sort_by_key(|span| span.start());

    Ok(BindingRenameParts {
        form,
        form_span: view.span,
        binding_span: target_parameter.name_span,
        binding_edit: target_parameter.binding_edit,
        reference_spans,
        shadowed_scope_count,
    })
}

fn iteration_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let binding_form = view
        .children
        .get(1)
        .with_context(|| format!("selected {form} form must contain an iteration binding"))?;
    let target = binding_form
        .children
        .first()
        .filter(|child| atom_text(child) == Some(from.as_str()))
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    if let Some(result_form) = binding_form.children.get(2) {
        collect_symbol_atom_spans_unshadowed(
            result_form,
            from,
            &mut reference_spans,
            &mut shadowed_scope_count,
            input,
        );
    }
    for body in &view.children[2..] {
        collect_symbol_atom_spans_unshadowed(
            body,
            from,
            &mut reference_spans,
            &mut shadowed_scope_count,
            input,
        );
    }
    reference_spans.sort_by_key(|span| span.start());

    Ok(BindingRenameParts {
        form,
        form_span: view.span,
        binding_span: target.span,
        binding_edit: types::BindingEdit::rename_atom(target.span),
        reference_spans,
        shadowed_scope_count,
    })
}

fn slot_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let slot_specs = view
        .children
        .get(1)
        .with_context(|| format!("selected {form} form must contain slot specs"))?;
    let mut target = None;
    let mut duplicate_count = 0usize;

    for spec in &slot_specs.children {
        let Some((name, span, edit)) = slot_spec_binding_name(spec) else {
            continue;
        };
        if name != from.as_str() {
            continue;
        }
        duplicate_count += 1;
        target = Some((span, edit));
    }

    if duplicate_count > 1 {
        anyhow::bail!(
            "binding '{from}' was found in multiple selected {form} specs; select an unambiguous binding form"
        );
    }

    let (binding_span, binding_edit) = target
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    for body in &view.children[3..] {
        collect_symbol_atom_spans_unshadowed(
            body,
            from,
            &mut reference_spans,
            &mut shadowed_scope_count,
            input,
        );
    }
    reference_spans.sort_by_key(|span| span.start());

    Ok(BindingRenameParts {
        form,
        form_span: view.span,
        binding_span,
        binding_edit,
        reference_spans,
        shadowed_scope_count,
    })
}

fn slot_spec_binding_name(spec: &ExpressionView) -> Option<(&str, ByteSpan, types::BindingEdit)> {
    match &spec.kind {
        ExpressionKind::Atom => {
            let name = atom_text(spec)?;
            Some((
                name,
                spec.span,
                types::BindingEdit::bare_slot_spec(spec.span, name.to_owned()),
            ))
        }
        ExpressionKind::List => {
            let first = spec.children.first()?;
            let name = atom_text(first)?;
            Some((
                name,
                first.span,
                types::BindingEdit::rename_atom(first.span),
            ))
        }
        ExpressionKind::Root => None,
    }
}
