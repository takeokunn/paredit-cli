use anyhow::{Context, Result};

use crate::domain::sexpr::SyntaxTree;

use super::calls::{add_function_parameter_call_edit, resolve_function_call_paths};
use super::definition::parse_add_function_parameter_definition;
use super::list_edit::{
    apply_byte_span_edits, ensure_non_overlapping_spans, insertion_edit_for_list_item,
    spans_overlap,
};
use super::types::{AddFunctionParameterPlan, AddFunctionParameterRequest};

pub fn plan_add_function_parameter(
    request: AddFunctionParameterRequest<'_>,
) -> Result<AddFunctionParameterPlan> {
    let argument = request.argument.trim().to_owned();
    if argument.is_empty() {
        anyhow::bail!("--argument must not be empty");
    }
    let argument_tree = SyntaxTree::parse(&argument)
        .context("add-function-parameter argument is not a valid S-expression")?;
    if argument_tree.root_children().len() != 1 {
        anyhow::bail!("--argument must contain exactly one top-level S-expression");
    }

    let tree = SyntaxTree::parse(request.input)?;
    let definition_selection = tree.select_path(&request.definition_path)?;
    let target = parse_add_function_parameter_definition(
        request.dialect,
        definition_selection.view(),
        &request.name,
    )?;
    if target.has_lambda_list_marker {
        anyhow::bail!(
            "add-function-parameter currently supports only flat positional parameter lists"
        );
    }
    let call_paths = resolve_function_call_paths(
        &tree,
        request.dialect,
        request.call_paths,
        request.all_calls,
        target.definition_span,
        &target.function_name,
        "add-function-parameter",
    )?;

    let mut edits = Vec::with_capacity(call_paths.len() + 1);
    edits.push(insertion_edit_for_list_item(
        &target.parameter_container,
        target.protected_prefix_count,
        request.name.as_str(),
        request.insert,
    )?);

    let mut call_spans = Vec::with_capacity(call_paths.len());
    for call_path in &call_paths {
        let call_selection = tree.select_path(call_path)?;
        if spans_overlap(target.definition_span, call_selection.span()) {
            anyhow::bail!(
                "add-function-parameter call path {} overlaps the selected definition",
                call_path
            );
        }
        let edit = add_function_parameter_call_edit(
            call_selection.view(),
            &target.function_name,
            &argument,
            request.insert,
        )?;
        call_spans.push(call_selection.span());
        edits.push(edit);
    }

    let mut sorted_call_spans = call_spans.clone();
    sorted_call_spans.sort_by_key(|span| span.start());
    ensure_non_overlapping_spans(sorted_call_spans)?;
    let rewritten = apply_byte_span_edits(request.input, edits)?;
    SyntaxTree::parse(&rewritten)
        .context("add-function-parameter output is not a valid S-expression document")?;

    let changed = rewritten != request.input;
    Ok(AddFunctionParameterPlan {
        dialect: request.dialect,
        definition_path: request.definition_path,
        call_paths,
        all_calls: request.all_calls,
        definition_span: target.definition_span,
        parameter_list_span: target.parameter_container.span,
        call_spans,
        function_name: target.function_name,
        parameter_name: request.name,
        argument,
        insert: request.insert,
        rewritten,
        changed,
    })
}
