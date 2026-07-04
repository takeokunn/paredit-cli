//! Use case for converting threading pipelines back into nested calls.

#[cfg(test)]
mod tests;

mod pipeline;
mod rewrite;
mod syntax;
mod types;

pub use types::{
    UnthreadExpressionPlan, UnthreadExpressionRequest, UnthreadExpressionStep, UnthreadStyle,
};

use crate::domain::sexpr::{Delimiter, ExpressionKind, SymbolName, SyntaxTree};
use anyhow::{Context, Result};
use pipeline::pipeline_step;
use rewrite::{replace_span, unthread_replacement};
use syntax::{atom_child, expression_source};

pub fn plan_unthread_expression(
    request: UnthreadExpressionRequest<'_>,
) -> Result<UnthreadExpressionPlan> {
    if request.target.kind != ExpressionKind::List
        || request.target.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!("unthread-expression target must be a parenthesized threading pipeline");
    }

    let head = atom_child(&request.target, 0)
        .context("unthread-expression target must start with an atom operator")?;
    if let Some(expected) = &request.operator {
        if head != expected.as_str() {
            anyhow::bail!(
                "unthread-expression operator mismatch: selected {head}, expected {expected}"
            );
        }
    }
    let operator = match request.operator {
        Some(operator) => operator,
        None => SymbolName::new(head)?,
    };
    let style = match (
        request.style,
        UnthreadStyle::from_operator(operator.as_str()),
    ) {
        (Some(style), _) => style,
        (None, Some(style)) => style,
        (None, None) => anyhow::bail!(
            "unthread-expression custom operator {} requires --style",
            operator
        ),
    };

    if request.target.children.len() < 3 {
        anyhow::bail!("unthread-expression pipeline must contain a base and at least one step");
    }

    let base_view = &request.target.children[1];
    let base = expression_source(request.input, base_view);
    let pipeline_steps = request
        .target
        .children
        .iter()
        .skip(2)
        .map(|view| pipeline_step(request.input, view))
        .collect::<Result<Vec<_>>>()?;
    let (replacement, steps) = unthread_replacement(style, &base, pipeline_steps);
    SyntaxTree::parse(&replacement).context("unthread-expression replacement does not parse")?;
    let rewritten = replace_span(request.input, request.target.span, &replacement);
    SyntaxTree::parse(&rewritten).context("unthread-expression rewritten output does not parse")?;
    let changed = rewritten != request.input;

    Ok(UnthreadExpressionPlan {
        dialect: request.dialect,
        path: request.path,
        style,
        operator,
        span: request.target.span,
        base,
        steps,
        replacement,
        rewritten,
        changed,
    })
}
