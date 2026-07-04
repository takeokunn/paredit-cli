//! Use-case helpers for inlining single-binding let forms.

mod parts;
mod rewrite;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

use anyhow::{Context, Result};

use crate::domain::sexpr::SyntaxTree;

use parts::inline_let_parts;
use rewrite::{replace_body_references, replace_span};
pub use types::{InlineLetPlan, InlineLetRequest};

pub fn plan_inline_let(request: InlineLetRequest<'_>) -> Result<InlineLetPlan> {
    let parts = inline_let_parts(request.dialect, request.input, &request.target)?;
    let reference_count = parts.reference_spans.len();
    if reference_count == 0 {
        anyhow::bail!("inline-let would drop an unused binding value");
    }
    if reference_count > 1 && !request.allow_duplicate_evaluation {
        anyhow::bail!(
            "inline-let would duplicate binding value evaluation; pass --allow-duplicate-evaluation to permit it"
        );
    }

    let replacement = replace_body_references(
        request.input,
        parts.body_span,
        &parts.reference_spans,
        &parts.binding_value,
    );
    let rewritten = replace_span(request.input, parts.let_span, &replacement);

    SyntaxTree::parse(&rewritten)
        .context("inline-let output is not a valid S-expression document")?;

    Ok(InlineLetPlan {
        dialect: request.dialect,
        path: request.path,
        let_span: parts.let_span,
        binding_name: parts.binding_name,
        binding_value: parts.binding_value,
        body_count: parts.body_count,
        reference_count,
        changed: rewritten != request.input,
        replacement,
        rewritten,
    })
}
