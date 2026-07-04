use std::fs;

use anyhow::{Context, Result};

use super::super::shared::{
    detect_dialect, list_head, package_context_before_top_level, read_input,
};
use super::args::RemoveDefinitionArgs;
use super::render::print_remove_definition_plan;
use super::types::RemoveDefinitionPlan;
use crate::application::usecase::definition_report::{
    body_form_count, count_lambda_parameters, definition_name, lambda_list_index,
    DefinitionReportItem,
};
use crate::domain::definition::classify_definition_head;
use crate::domain::sexpr::{Edit, SyntaxTree};

pub(in crate::presentation::cli) fn remove_definition(args: RemoveDefinitionArgs) -> Result<()> {
    let input = read_input(Some(args.file.clone()))?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)
        .with_context(|| format!("failed to parse {}", args.file.display()))?;

    let target_index = match args.path.indexes() {
        [index] => index.get(),
        _ => anyhow::bail!(
            "remove-definition requires a top-level definition path, for example --path 2"
        ),
    };
    if target_index >= tree.root_children().len() {
        anyhow::bail!("top-level path {} is out of range", args.path);
    }

    let selection = tree.select_path(&args.path)?;
    let view = selection.view();
    let span = selection.span();
    let Some(head) = list_head(&view) else {
        anyhow::bail!("selected top-level form is not a list definition");
    };
    let Some(category) = classify_definition_head(dialect, head) else {
        anyhow::bail!("selected top-level form is not recognized as a definition: {head}");
    };

    let definition_text = selection.text(&input.text).to_owned();
    let lambda_index = lambda_list_index(&view, head);
    let definition = DefinitionReportItem {
        path: args.path.to_string(),
        span,
        head: head.to_owned(),
        name: definition_name(&view, head).map(ToOwned::to_owned),
        category,
        parameter_count: lambda_index.map(|index| count_lambda_parameters(&view.children[index])),
        body_form_count: body_form_count(&view, lambda_index),
        package: package_context_before_top_level(&tree, target_index)?,
    };
    let rewritten = Edit::kill(&input.text, &tree, selection)?;

    SyntaxTree::parse(&rewritten).with_context(|| {
        format!(
            "file would become invalid after removing definition: {}",
            args.file.display()
        )
    })?;

    let changed = rewritten != input.text;
    let written = args.write && changed;
    if written {
        fs::write(&args.file, &rewritten)
            .with_context(|| format!("failed to write {}", args.file.display()))?;
    }

    let plan = RemoveDefinitionPlan {
        file: args.file,
        dialect,
        path: args.path,
        span,
        definition,
        definition_text,
        rewritten,
        changed,
        written,
    };
    print_remove_definition_plan(&plan, args.output)
}
