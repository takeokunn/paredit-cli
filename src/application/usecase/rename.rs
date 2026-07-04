//! Use-case helpers for renaming callable definitions and wrapping call sites.

mod binding;
mod function;
mod selection;
#[cfg(test)]
mod tests;
mod types;
mod wrap;

use anyhow::{Context, Result};

use crate::domain::sexpr::SyntaxTree;

use binding::binding_rename_parts;
use selection::{apply_byte_span_edits, collect_symbol_atom_spans, select_rename_target};

pub use function::{collect_callable_definition_renames, collect_function_call_head_renames};
pub use types::{
    RenameBindingPlan, RenameBindingRequest, RenameFunctionOccurrence, RenameFunctionPlan,
    RenameFunctionRequest, RenameInFormPlan, RenameInFormRequest, RenameTarget,
};
pub use wrap::{
    WrapFunctionCallSite, WrapFunctionCallsPlan, WrapFunctionCallsRequest, WrapFunctionCallsScope,
    plan_wrap_function_calls,
};

pub fn plan_rename_function(request: RenameFunctionRequest<'_>) -> Result<RenameFunctionPlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let definitions =
        collect_callable_definition_renames(&tree, request.dialect, &request.from, &request.to)?;
    let calls =
        collect_function_call_head_renames(&tree, request.dialect, &request.from, &request.to)?;
    let edits = definitions
        .iter()
        .chain(calls.iter())
        .map(|occurrence| (occurrence.span, occurrence.replacement.clone()))
        .collect::<Vec<_>>();
    let rewritten = apply_byte_span_edits(request.input, edits)?;
    SyntaxTree::parse(&rewritten).context("renamed output is not a valid S-expression document")?;

    Ok(RenameFunctionPlan {
        dialect: request.dialect,
        definitions,
        calls,
        changed: rewritten != request.input,
        rewritten,
    })
}

pub fn plan_rename_in_form(request: RenameInFormRequest<'_>) -> Result<RenameInFormPlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let path = match &request.target {
        RenameTarget::Path(path) => Some(path.clone()),
        RenameTarget::Offset(_) => None,
    };
    let view = select_rename_target(&tree, &request.target)?.view();
    let mut occurrences = Vec::new();
    collect_symbol_atom_spans(&view, &request.from, &mut occurrences);
    occurrences.sort_by_key(|span| span.start());

    let edits = occurrences
        .iter()
        .map(|span| (*span, request.to.as_str().to_owned()))
        .collect::<Vec<_>>();
    let rewritten = apply_byte_span_edits(request.input, edits)?;
    SyntaxTree::parse(&rewritten).context("renamed output is not a valid S-expression document")?;

    Ok(RenameInFormPlan {
        dialect: request.dialect,
        path,
        scope_span: view.span,
        from: request.from,
        to: request.to,
        occurrences,
        changed: rewritten != request.input,
        rewritten,
    })
}

pub fn plan_rename_binding(request: RenameBindingRequest<'_>) -> Result<RenameBindingPlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let path = match &request.target {
        RenameTarget::Path(path) => Some(path.clone()),
        RenameTarget::Offset(_) => None,
    };
    let view = select_rename_target(&tree, &request.target)?.view();
    let parts = binding_rename_parts(request.dialect, &view, &request.from, request.input)?;

    let mut edits = Vec::with_capacity(parts.reference_spans.len() + 1);
    edits.push((
        parts.binding_edit.span,
        parts.binding_edit.replacement(request.input, &request.to),
    ));
    edits.extend(
        parts
            .reference_spans
            .iter()
            .map(|span| (*span, request.to.as_str().to_owned())),
    );
    let rewritten = apply_byte_span_edits(request.input, edits)?;
    SyntaxTree::parse(&rewritten).context("renamed output is not a valid S-expression document")?;

    Ok(RenameBindingPlan {
        dialect: request.dialect,
        path,
        form: parts.form,
        form_span: parts.form_span,
        binding_span: parts.binding_span,
        from: request.from,
        to: request.to,
        references: parts.reference_spans,
        shadowed_scope_count: parts.shadowed_scope_count,
        changed: rewritten != request.input,
        rewritten,
    })
}
