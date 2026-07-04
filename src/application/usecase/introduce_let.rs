//! Use case for introducing a local binding around a selected expression.

mod occurrences;
mod rewrite;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

use anyhow::{Context, Result, bail};

use crate::domain::sexpr::{ByteOffset, ByteSpan, Path, SyntaxTree};

use occurrences::{
    EquivalentExpressionSpans, collect_equivalent_expression_spans, is_span_shadowed_by_binding,
    rebase_spans,
};
use rewrite::{introduced_let, replace_span, replace_spans_within_span};
pub use types::{IntroduceLetPlan, IntroduceLetRequest};

pub fn plan_introduce_let(request: IntroduceLetRequest<'_>) -> Result<IntroduceLetPlan> {
    let selected_span = request.target.span;
    let binding_value = selected_span.slice(request.input).to_owned();
    let enclosing = request.enclosing_span.slice(request.input);
    let enclosing_tree =
        SyntaxTree::parse(enclosing).context("failed to parse enclosing list for introduce-let")?;
    let enclosing_view = enclosing_tree
        .select_path(&Path::from_indexes(vec![0]))?
        .view();

    let selected_relative_span = ByteSpan::new(
        ByteOffset::new(selected_span.start().get() - request.enclosing_span.start().get()),
        ByteOffset::new(selected_span.end().get() - request.enclosing_span.start().get()),
    );

    let (occurrence_spans, skipped_shadowed_occurrence_spans) = if request.all_occurrences {
        let mut collection = EquivalentExpressionSpans::default();
        collect_equivalent_expression_spans(
            &enclosing_view,
            &request.target,
            request.name.as_str(),
            false,
            &mut collection,
        );
        (
            rebase_spans(collection.replacement_spans, request.enclosing_span.start()),
            rebase_spans(
                collection.skipped_shadowed_spans,
                request.enclosing_span.start(),
            ),
        )
    } else {
        if is_span_shadowed_by_binding(
            &enclosing_view,
            selected_relative_span,
            request.name.as_str(),
            false,
        ) {
            bail!(
                "introduce-let target is inside an existing binding for '{}'; choose a different --name",
                request.name.as_str()
            );
        }
        (vec![selected_span], Vec::new())
    };

    if !occurrence_spans.contains(&selected_span) {
        bail!(
            "introduce-let target is inside an existing binding for '{}'; choose a different --name",
            request.name.as_str()
        );
    }

    let enclosed_replacement = replace_spans_within_span(
        request.input,
        request.enclosing_span,
        &occurrence_spans,
        request.name.as_str(),
    );
    let replacement = introduced_let(
        request.dialect,
        &request.name,
        &binding_value,
        &enclosed_replacement,
    );
    let rewritten = replace_span(request.input, request.enclosing_span, &replacement);

    SyntaxTree::parse(&rewritten)
        .context("introduced-let output is not a valid S-expression document")?;

    let changed = rewritten != request.input;

    Ok(IntroduceLetPlan {
        dialect: request.dialect,
        path: request.path,
        selected_span,
        enclosing_span: request.enclosing_span,
        name: request.name,
        binding_value,
        occurrence_spans,
        skipped_shadowed_occurrence_spans,
        replacement,
        rewritten,
        changed,
    })
}
