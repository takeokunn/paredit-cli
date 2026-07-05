mod destructure;
mod forms;
mod rewrite;
mod scope;
mod types;

use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, SymbolName};

use super::selection::{atom_text, list_head};
use destructure::binding_pattern_name_spans;
use forms::{binding_groups, parameter_name_spans, specialized_parameter_name_spans};
use scope::collect_symbol_atom_spans_unshadowed;
use types::BindingEdit;
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
        "defmethod" | "cl-defmethod" => defmethod_binding_rename_parts(view, from, form, input),
        "defun" | "defmacro" | "define-setf-expander" | "define-compiler-macro" => {
            parameter_binding_rename_parts(view, from, form, 2, 3, input)
        }
        "flet" | "labels" | "macrolet" | "compiler-macrolet" => {
            local_callable_lambda_binding_rename_parts(view, from, form, input)
        }
        "handler-case" | "restart-case" => clause_binding_rename_parts(view, from, form, input),
        "handler-bind" | "restart-bind" => {
            handler_bind_lambda_binding_rename_parts(view, from, form, input)
        }
        "dolist" | "dotimes" => iteration_binding_rename_parts(view, from, form, input),
        "loop" => loop_binding_rename_parts(view, from, form, input),
        "do" | "do*" | "prog" | "prog*" => {
            common_lisp_variable_binding_rename_parts(view, from, form, input)
        }
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
    if form == "symbol-macrolet" && bindings.iter().any(|binding| binding.value.is_none()) {
        anyhow::bail!("symbol-macrolet binding must contain a symbol and expansion");
    }
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
            if let Some(value) = &later.value {
                collect_symbol_atom_spans_unshadowed(
                    value,
                    from,
                    &mut reference_spans,
                    &mut shadowed_scope_count,
                    input,
                );
            }
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

fn defmethod_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let parameter_index = defmethod_specialized_lambda_list_index(view)
        .with_context(|| format!("selected {form} form must contain a specialized lambda list"))?;
    let parameter_form = &view.children[parameter_index];
    let parameters = specialized_parameter_name_spans(parameter_form, input)?;
    let target = parameters
        .iter()
        .find(|parameter| parameter.name == from.as_str())
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    for body in &view.children[parameter_index + 1..] {
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

fn defmethod_specialized_lambda_list_index(view: &ExpressionView) -> Option<usize> {
    view.children
        .iter()
        .enumerate()
        .skip(2)
        .find_map(|(index, child)| {
            (child.kind == ExpressionKind::List && child.delimiter == Some(Delimiter::Paren))
                .then_some(index)
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

fn local_callable_lambda_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let binding_form = view
        .children
        .get(1)
        .with_context(|| format!("selected {form} form must contain local callable bindings"))?;
    let mut target = None;
    let mut duplicate_count = 0usize;

    for binding in &binding_form.children {
        if binding.kind != ExpressionKind::List || binding.delimiter != Some(Delimiter::Paren) {
            continue;
        }

        let Some(parameter_form) = binding.children.get(1) else {
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
        target = Some((binding, parameter.clone()));
    }

    if duplicate_count > 1 {
        anyhow::bail!(
            "binding '{from}' was found in multiple selected {form} local callable lambda lists; select an unambiguous binding form"
        );
    }

    let (target_binding, target_parameter) = target.ok_or_else(|| {
        anyhow::anyhow!(
            "binding '{from}' was not found in selected {form} local callable lambda lists"
        )
    })?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    for body in &target_binding.children[2..] {
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

fn handler_bind_lambda_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let mut target = None;
    let mut duplicate_count = 0usize;

    for function_form in handler_bind_function_forms(view, form.as_str()) {
        collect_lambda_binding_targets(
            function_form,
            from,
            input,
            &mut target,
            &mut duplicate_count,
        )?;
    }

    if duplicate_count > 1 {
        anyhow::bail!(
            "binding '{from}' was found in multiple selected {form} handler functions; select an unambiguous binding form"
        );
    }

    let (target_lambda, target_parameter) = target
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    for body in &target_lambda.children[2..] {
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

fn loop_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let mut target = None;
    let mut duplicate_count = 0usize;

    for spec in loop_binding_specs(view, input) {
        if spec.name != from.as_str() {
            continue;
        }
        duplicate_count += 1;
        target = Some(spec.clone());
    }

    if duplicate_count > 1 {
        anyhow::bail!(
            "binding '{from}' was found in multiple selected {form} clauses; select an unambiguous binding form"
        );
    }

    let target = target
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    for body in &view.children[target.reference_start_index..] {
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
        binding_edit: target.binding_edit,
        reference_spans,
        shadowed_scope_count,
    })
}

fn common_lisp_variable_binding_rename_parts(
    view: &ExpressionView,
    from: &SymbolName,
    form: String,
    input: &str,
) -> Result<BindingRenameParts> {
    let binding_form = view
        .children
        .get(1)
        .with_context(|| format!("selected {form} form must contain variable specs"))?;
    let mut target = None;
    let mut duplicate_count = 0usize;

    for (index, spec) in binding_form.children.iter().enumerate() {
        let Some((name, span)) = common_lisp_variable_spec_binding_name(spec) else {
            continue;
        };
        if name != from.as_str() {
            continue;
        }
        duplicate_count += 1;
        target = Some((index, span));
    }

    if duplicate_count > 1 {
        anyhow::bail!(
            "binding '{from}' was found in multiple selected {form} specs; select an unambiguous binding form"
        );
    }

    let (target_index, binding_span) = target
        .ok_or_else(|| anyhow::anyhow!("binding '{from}' was not found in selected {form}"))?;

    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    if matches!(form.as_str(), "do*" | "prog*") {
        for spec in binding_form.children.iter().skip(target_index + 1) {
            if let Some(init_form) = common_lisp_variable_spec_init_form(spec) {
                collect_symbol_atom_spans_unshadowed(
                    init_form,
                    from,
                    &mut reference_spans,
                    &mut shadowed_scope_count,
                    input,
                );
            }
        }
    }

    if matches!(form.as_str(), "do" | "do*") {
        for spec in &binding_form.children {
            if let Some(step_form) = common_lisp_do_variable_spec_step_form(spec) {
                collect_symbol_atom_spans_unshadowed(
                    step_form,
                    from,
                    &mut reference_spans,
                    &mut shadowed_scope_count,
                    input,
                );
            }
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
        binding_span,
        binding_edit: types::BindingEdit::rename_atom(binding_span),
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

#[derive(Clone)]
struct LoopBindingSpec {
    name: String,
    name_span: ByteSpan,
    binding_edit: BindingEdit,
    reference_start_index: usize,
}

fn loop_binding_specs(view: &ExpressionView, input: &str) -> Vec<LoopBindingSpec> {
    let mut specs = Vec::new();
    let mut index = 1usize;

    while index < view.children.len() {
        let child = &view.children[index];
        if loop_keyword_is(child, "for") || loop_keyword_is(child, "as") {
            if let Some(name_form) = view.children.get(index + 1) {
                let reference_start_index =
                    loop_for_reference_start_index(&view.children, index + 2);
                push_loop_binding_specs(&mut specs, name_form, reference_start_index, input);
            }
            index += 2;
            continue;
        }

        if loop_keyword_is(child, "with") {
            if let Some(name_form) = view.children.get(index + 1) {
                let reference_start_index =
                    loop_with_reference_start_index(&view.children, index + 2);
                push_loop_binding_specs(&mut specs, name_form, reference_start_index, input);
            }
            index += 2;
            continue;
        }

        index += 1;
    }

    specs
}

fn push_loop_binding_specs(
    specs: &mut Vec<LoopBindingSpec>,
    name_form: &ExpressionView,
    reference_start_index: usize,
    input: &str,
) {
    specs.extend(
        binding_pattern_name_spans(name_form, input)
            .into_iter()
            .map(|name| LoopBindingSpec {
                name: name.name,
                name_span: name.name_span,
                binding_edit: name.binding_edit,
                reference_start_index,
            }),
    );
}

fn loop_for_reference_start_index(children: &[ExpressionView], mut index: usize) -> usize {
    let Some(keyword) = children.get(index).and_then(atom_text) else {
        return index;
    };

    if matches_loop_keyword(keyword, &["in", "on", "across"]) {
        return (index + 2).min(children.len());
    }

    if matches_loop_keyword(keyword, &["=", "from", "downfrom", "upfrom"]) {
        index = (index + 2).min(children.len());
        while children.get(index).and_then(atom_text).is_some_and(|text| {
            matches_loop_keyword(text, &["to", "upto", "downto", "below", "above", "by"])
        }) {
            index = (index + 2).min(children.len());
        }
    }

    index
}

fn loop_with_reference_start_index(children: &[ExpressionView], index: usize) -> usize {
    if children
        .get(index)
        .is_some_and(|child| loop_keyword_is(child, "="))
    {
        return (index + 2).min(children.len());
    }

    index
}

fn loop_keyword_is(view: &ExpressionView, keyword: &str) -> bool {
    atom_text(view).is_some_and(|text| text.eq_ignore_ascii_case(keyword))
}

fn matches_loop_keyword(text: &str, keywords: &[&str]) -> bool {
    keywords
        .iter()
        .any(|keyword| text.eq_ignore_ascii_case(keyword))
}

fn common_lisp_variable_spec_binding_name(spec: &ExpressionView) -> Option<(&str, ByteSpan)> {
    match &spec.kind {
        ExpressionKind::Atom => Some((atom_text(spec)?, spec.span)),
        ExpressionKind::List => {
            let first = spec.children.first()?;
            Some((atom_text(first)?, first.span))
        }
        ExpressionKind::Root => None,
    }
}

fn common_lisp_variable_spec_init_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    (spec.kind == ExpressionKind::List)
        .then(|| spec.children.get(1))
        .flatten()
}

fn common_lisp_do_variable_spec_step_form(spec: &ExpressionView) -> Option<&ExpressionView> {
    (spec.kind == ExpressionKind::List)
        .then(|| spec.children.get(2))
        .flatten()
}

fn handler_bind_function_forms<'a>(
    view: &'a ExpressionView,
    form: &str,
) -> Vec<&'a ExpressionView> {
    let Some(binding_form) = view.children.get(1) else {
        return Vec::new();
    };

    let mut forms = Vec::new();
    for spec in &binding_form.children {
        if spec.kind != ExpressionKind::List || spec.delimiter != Some(Delimiter::Paren) {
            continue;
        }

        if let Some(function_form) = spec.children.get(1) {
            forms.push(function_form);
        }

        if form == "restart-bind" {
            let mut index = 2usize;
            while index + 1 < spec.children.len() {
                forms.push(&spec.children[index + 1]);
                index += 2;
            }
        }
    }

    forms
}

fn collect_lambda_binding_targets<'a>(
    view: &'a ExpressionView,
    from: &SymbolName,
    input: &str,
    target: &mut Option<(&'a ExpressionView, types::ParameterNameSpan)>,
    duplicate_count: &mut usize,
) -> Result<()> {
    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && atom_text(view.children.first().unwrap_or(view)) == Some("lambda")
    {
        if let Some(parameter_form) = view.children.get(1) {
            let parameters = parameter_name_spans(parameter_form, input)?;
            if let Some(parameter) = parameters
                .iter()
                .find(|parameter| parameter.name == from.as_str())
            {
                *duplicate_count += 1;
                *target = Some((view, parameter.clone()));
            }
        }
    }

    for child in &view.children {
        collect_lambda_binding_targets(child, from, input, target, duplicate_count)?;
    }

    Ok(())
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
