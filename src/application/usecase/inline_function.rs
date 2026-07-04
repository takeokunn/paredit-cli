//! Use-case helpers for inlining function calls.

use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, ExpressionView, SymbolName, SyntaxTree};

mod calls;
mod definition;
mod rewrite;
mod substitution;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

use calls::{
    bind_inline_function_arguments, parse_inline_function_call, resolve_function_call_paths,
};
use definition::parse_inline_function_definition;
use rewrite::{apply_byte_span_edits, expand_definition_removal};
use substitution::substitute_inline_function_body;
use syntax::spans_overlap;
pub use types::{
    InlineFunctionCallPlan, InlineFunctionParameterPlan, InlineFunctionPlan, InlineFunctionRequest,
};

#[derive(Debug)]
struct InlineFunctionParts {
    definition_span: ByteSpan,
    call_span: ByteSpan,
    function_name: SymbolName,
    parameters: Vec<InlineFunctionParameterPlan>,
    replacement: String,
}

pub fn plan_inline_function(request: InlineFunctionRequest<'_>) -> Result<InlineFunctionPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    let definition_selection = tree.select_path(&request.definition_path)?;
    let definition_span = definition_selection.span();
    let (function_name, _, _) =
        parse_inline_function_definition(request.dialect, definition_selection.view())?;
    let call_paths = resolve_function_call_paths(
        &tree,
        request.dialect,
        request.call_paths,
        request.all_calls,
        definition_span,
        &function_name,
        "inline-function",
    )?;

    let mut calls = Vec::with_capacity(call_paths.len());
    let mut edits = Vec::with_capacity(call_paths.len() + usize::from(request.remove_definition));
    for call_path in &call_paths {
        let definition_selection = tree.select_path(&request.definition_path)?;
        let call_selection = tree.select_path(call_path)?;
        let parts = inline_function_parts(
            request.dialect,
            request.input,
            definition_selection.view(),
            call_selection.view(),
            request.allow_duplicate_evaluation,
            request.allow_drop_arguments,
        )?;

        if parts.function_name != function_name {
            anyhow::bail!(
                "inline-function resolved inconsistent function name: expected {}, found {}",
                function_name,
                parts.function_name
            );
        }
        if spans_overlap(parts.definition_span, parts.call_span) {
            anyhow::bail!("inline-function definition and call selections must not overlap");
        }

        edits.push((parts.call_span, parts.replacement.clone()));
        calls.push(InlineFunctionCallPlan {
            call_path: call_path.clone(),
            call_span: parts.call_span,
            parameters: parts.parameters,
            replacement: parts.replacement,
        });
    }

    let call_spans = calls.iter().map(|call| call.call_span).collect::<Vec<_>>();
    let definition_removed = request.remove_definition;
    if definition_removed {
        edits.push((
            expand_definition_removal(request.input, definition_span),
            String::new(),
        ));
    }
    let rewritten = apply_byte_span_edits(request.input, edits)?;

    SyntaxTree::parse(&rewritten)
        .context("inline-function output is not a valid S-expression document")?;

    Ok(InlineFunctionPlan {
        dialect: request.dialect,
        definition_path: request.definition_path,
        call_paths,
        all_calls: request.all_calls,
        definition_span,
        call_spans,
        function_name,
        calls,
        remove_definition: request.remove_definition,
        definition_removed,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn inline_function_parts(
    dialect: Dialect,
    input: &str,
    definition_selection: ExpressionView,
    call_selection: ExpressionView,
    allow_duplicate_evaluation: bool,
    allow_drop_arguments: bool,
) -> Result<InlineFunctionParts> {
    let (function_name, params, body) =
        parse_inline_function_definition(dialect, definition_selection.clone())?;
    let raw_args = parse_inline_function_call(call_selection.clone(), &function_name, input)?;
    let args = bind_inline_function_arguments(&params, raw_args, &function_name)?;
    let param_names = params
        .iter()
        .map(|param| param.name.clone())
        .collect::<Vec<_>>();

    let (replacement, parameters) = substitute_inline_function_body(
        input,
        &body,
        &param_names,
        &args,
        allow_duplicate_evaluation,
        allow_drop_arguments,
    )?;

    Ok(InlineFunctionParts {
        definition_span: definition_selection.span,
        call_span: call_selection.span,
        function_name,
        parameters,
        replacement,
    })
}
