//! Use-case helpers for extracting functions from selected expressions.

use anyhow::{Context, Result};

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::sexpr::{ExpressionView, SyntaxTree};

mod inference;
mod rewrite;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

use crate::application::usecase::extract_shared::{insert_top_level_form, replace_span};
use rewrite::{extracted_call, extracted_definition};

pub use types::{ExtractFunctionInsert, ExtractFunctionPlan, ExtractFunctionRequest};

pub fn plan_extract_function(request: ExtractFunctionRequest<'_>) -> Result<ExtractFunctionPlan> {
    let input_tree = SyntaxTree::parse(request.input)
        .context("extract-function input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&input_tree, request.dialect)?;

    let span = request.selection.span();
    let selected = request.selection.text(request.input).to_owned();
    let mut params = request.explicit_params;
    let inferred_params = if request.infer_params {
        infer_extract_function_params(request.dialect, &request.selection.view(), &params)
    } else {
        Vec::new()
    };
    for param in &inferred_params {
        if !params.iter().any(|existing| {
            inference::extract_function_param_name_eq(request.dialect, existing, param)
        }) {
            params.push(param.clone());
        }
    }

    let call = extracted_call(&request.name, &params);
    let definition = extracted_definition(request.dialect, &request.name, &params, &selected);
    let replaced = replace_span(request.input, span, &call);
    let replaced_tree = SyntaxTree::parse(&replaced)
        .context("replacement output is not a valid S-expression document")?;
    let (rewritten, anchor_span) = insert_top_level_form(
        &replaced,
        &replaced_tree,
        &definition,
        request.insert,
        request.anchor_path.as_ref(),
        "extract-function --anchor-path",
    )?;

    SyntaxTree::parse(&rewritten)
        .context("extracted output is not a valid S-expression document")?;

    Ok(ExtractFunctionPlan {
        dialect: request.dialect,
        path: request.path,
        span_start: span.start().get(),
        span_end: span.end().get(),
        name: request.name,
        params,
        inferred_params,
        insert: request.insert,
        anchor_path: request.anchor_path,
        anchor_span,
        call,
        definition,
        changed: rewritten != request.input,
        rewritten,
    })
}

pub fn infer_extract_function_params(
    dialect: crate::domain::dialect::Dialect,
    selection: &ExpressionView,
    explicit_params: &[String],
) -> Vec<String> {
    inference::infer_extract_function_params(dialect, selection, explicit_params)
}
