use anyhow::Result;

use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{ExpressionView, Path, SymbolName, SyntaxTree};

use super::InlineFunctionParameterPlan;
use super::rewrite::apply_relative_body_edits;

pub(super) fn substitute_inline_function_body(
    input: &str,
    body: &ExpressionView,
    params: &[String],
    args: &[String],
    allow_duplicate_evaluation: bool,
    allow_drop_arguments: bool,
) -> Result<(String, Vec<InlineFunctionParameterPlan>)> {
    substitute_references(
        input,
        body,
        params,
        args,
        allow_duplicate_evaluation,
        allow_drop_arguments,
    )
}

pub(super) fn substitute_expression(
    input: &str,
    params: &[String],
    args: &[String],
) -> Result<String> {
    let tree = SyntaxTree::parse(input)?;
    if tree.root_children().len() != 1 {
        anyhow::bail!("inline-function default value must be a single S-expression");
    }
    let expression = tree.select_path(&Path::root_child(0))?.view();
    let (rewritten, _) = substitute_references(input, &expression, params, args, true, true)?;
    Ok(rewritten)
}

fn substitute_references(
    input: &str,
    body: &ExpressionView,
    params: &[String],
    args: &[String],
    allow_duplicate_evaluation: bool,
    allow_drop_arguments: bool,
) -> Result<(String, Vec<InlineFunctionParameterPlan>)> {
    let mut replacements = Vec::new();
    let mut parameter_plans = Vec::with_capacity(params.len());

    for (param, argument) in params.iter().zip(args) {
        let symbol = SymbolName::new(param.clone())?;
        let mut spans = Vec::new();
        collect_unshadowed_symbol_references(body, &symbol, input, &mut spans);
        spans.sort_by_key(|span| span.start());

        if spans.is_empty() && !allow_drop_arguments {
            anyhow::bail!(
                "inline-function would drop argument '{}' for unused parameter '{}'; pass --allow-drop-arguments to permit it",
                argument,
                param
            );
        }
        if spans.len() > 1 && !allow_duplicate_evaluation {
            anyhow::bail!(
                "inline-function would duplicate argument '{}' for parameter '{}'; pass --allow-duplicate-evaluation to permit it",
                argument,
                param
            );
        }

        for span in &spans {
            replacements.push((*span, argument.clone()));
        }
        parameter_plans.push(InlineFunctionParameterPlan {
            name: param.clone(),
            argument: argument.clone(),
            reference_count: spans.len(),
        });
    }

    Ok((
        apply_relative_body_edits(input, body.span, replacements)?,
        parameter_plans,
    ))
}
