//! Use-case helpers for removing unused let bindings.

use anyhow::{Context, Result};

use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, SymbolName, SyntaxTree};

mod candidates;
mod references;
mod rewrite;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

use candidates::{
    let_binding_removal_candidates, local_callable_binding_removal_candidates,
    macrolet_binding_removal_candidates,
};
use references::{let_binding_reference_spans, local_callable_binding_reference_spans};
use rewrite::{apply_nested_span_edits, replace_span};
use syntax::atom_text;
use types::{RemoveUnusedBindingParts, RemovedBindingParts};
pub use types::{RemoveUnusedBindingPlan, RemoveUnusedBindingRequest, RemovedBindingPlan};

pub fn plan_remove_unused_binding(
    request: RemoveUnusedBindingRequest<'_>,
) -> Result<RemoveUnusedBindingPlan> {
    if request.name.is_some() && request.all_bindings {
        anyhow::bail!("remove-unused-binding accepts either --name or --all-bindings, not both");
    }
    if request.name.is_none() && !request.all_bindings {
        anyhow::bail!("remove-unused-binding requires --name or --all-bindings");
    }

    let parts = remove_unused_binding_parts(
        request.dialect,
        request.input,
        &request.target,
        request.name,
        request.all_bindings,
    )?;
    let rewritten = replace_span(request.input, parts.form_span, &parts.replacement);
    SyntaxTree::parse(&rewritten)
        .context("remove-unused-binding output is not a valid S-expression document")?;

    let bindings = parts
        .bindings
        .iter()
        .map(|binding| RemovedBindingPlan {
            binding_name: binding.name.clone(),
            binding_span: binding.binding_span,
            binding_value: binding.binding_value.clone(),
            reference_count: binding.reference_spans.len(),
        })
        .collect::<Vec<_>>();
    let first_binding = bindings.first();

    Ok(RemoveUnusedBindingPlan {
        dialect: request.dialect,
        path: request.path,
        form: parts.form,
        form_span: parts.form_span,
        binding_name: first_binding.map(|binding| binding.binding_name.clone()),
        binding_span: first_binding.map(|binding| binding.binding_span),
        binding_value: first_binding.map(|binding| binding.binding_value.clone()),
        reference_count: first_binding.map(|binding| binding.reference_count),
        bindings,
        dropped_value_requires_review: !request.allow_drop_value,
        replacement: parts.replacement,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn remove_unused_binding_parts(
    dialect: crate::domain::dialect::Dialect,
    input: &str,
    target: &ExpressionView,
    name: Option<&SymbolName>,
    all_bindings: bool,
) -> Result<RemoveUnusedBindingParts> {
    if target.kind != ExpressionKind::List || target.delimiter != Some(Delimiter::Paren) {
        anyhow::bail!(
            "remove-unused-binding selection must be a let, let*, symbol-macrolet, flet, labels, macrolet, or compiler-macrolet list"
        );
    }
    if target.children.len() < 3 {
        anyhow::bail!(
            "remove-unused-binding requires a let, let*, symbol-macrolet, flet, labels, macrolet, or compiler-macrolet form with bindings and a body"
        );
    }
    let head = atom_text(&target.children[0])
        .context("remove-unused-binding form must start with an atom")?;
    if !matches!(
        head,
        "let" | "let*" | "symbol-macrolet" | "flet" | "labels" | "macrolet" | "compiler-macrolet"
    ) {
        anyhow::bail!(
            "remove-unused-binding selection must start with let, let*, symbol-macrolet, flet, labels, macrolet, or compiler-macrolet"
        );
    }

    let binding_form = &target.children[1];
    let candidates = if matches!(head, "macrolet" | "compiler-macrolet") {
        macrolet_binding_removal_candidates(dialect, binding_form)?
    } else if matches!(head, "flet" | "labels") {
        local_callable_binding_removal_candidates(dialect, binding_form)?
    } else {
        let_binding_removal_candidates(dialect, binding_form)?
    };
    let selected = if all_bindings {
        let mut unused = Vec::new();
        for candidate in &candidates {
            let symbol = SymbolName::new(candidate.name.clone())?;
            let reference_spans = if matches!(head, "flet" | "labels") {
                local_callable_binding_reference_spans(dialect, target, &symbol)?
            } else {
                let_binding_reference_spans(
                    input,
                    target,
                    binding_form,
                    &candidates,
                    candidate,
                    &symbol,
                )?
            };
            if reference_spans.is_empty() {
                unused.push(RemovedBindingParts {
                    name: candidate.name.clone(),
                    binding_span: candidate.removal_span,
                    binding_value: candidate.value_span.slice(input).to_owned(),
                    reference_spans,
                });
            }
        }
        if unused.is_empty() {
            anyhow::bail!("remove-unused-binding --all-bindings found no unused bindings");
        }
        unused
    } else {
        let name = name.expect("validated by caller");
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.name == name.as_str())
            .with_context(|| {
                format!(
                    "binding {} was not found in selected binding form",
                    name.as_str()
                )
            })?;
        let reference_spans = if matches!(head, "flet" | "labels") {
            local_callable_binding_reference_spans(dialect, target, name)?
        } else {
            let_binding_reference_spans(input, target, binding_form, &candidates, candidate, name)?
        };
        let reference_count = reference_spans.len();
        if reference_count != 0 {
            anyhow::bail!(
                "remove-unused-binding requires zero in-scope references; found {reference_count}"
            );
        }
        vec![RemovedBindingParts {
            name: candidate.name.clone(),
            binding_span: candidate.removal_span,
            binding_value: candidate.value_span.slice(input).to_owned(),
            reference_spans,
        }]
    };

    let replacement = if selected.len() == candidates.len() {
        let first_body = &target.children[2];
        let last_body = target
            .children
            .last()
            .expect("validated let form has at least one body expression");
        crate::domain::sexpr::ByteSpan::new(first_body.span.start(), last_body.span.end())
            .slice(input)
            .to_owned()
    } else {
        apply_nested_span_edits(
            target.span.slice(input),
            target.span,
            selected
                .iter()
                .map(|binding| (binding.binding_span, String::new()))
                .collect(),
        )?
    };

    Ok(RemoveUnusedBindingParts {
        form: head.to_owned(),
        form_span: target.span,
        bindings: selected,
        replacement,
    })
}
