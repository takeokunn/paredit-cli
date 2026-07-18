use anyhow::{Context, Result};

use super::super::shared::{
    list_head, package_context_before_top_level, read_input_dialect_and_tree,
    write_file_with_rollback,
};
use super::args::RemoveDefinitionArgs;
use super::render::print_remove_definition_plan;
use super::types::RemoveDefinitionPlan;
use crate::application::usecase::definition_report::DefinitionReportItem;
use crate::domain::definition::definition_shape;
use crate::domain::sexpr::{Edit, SyntaxTree};

pub(in crate::presentation::cli) fn remove_definition(args: RemoveDefinitionArgs) -> Result<()> {
    let (input, dialect, tree) =
        read_input_dialect_and_tree(Some(args.file.clone()), args.dialect)?;

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
    let Some(shape) = definition_shape(dialect, &view, head) else {
        anyhow::bail!("selected top-level form is not recognized as a definition: {head}");
    };

    let definition_text = selection.text().to_owned();
    let definition = DefinitionReportItem {
        path: args.path.to_string(),
        span,
        head: head.to_owned(),
        name: shape.name(&view).map(ToOwned::to_owned),
        category: shape.category,
        parameter_count: shape.lambda_parameter_count(&view),
        body_form_count: Some(shape.body_form_count(&view)),
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
        write_file_with_rollback(args.file.clone(), rewritten.clone())?;
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
