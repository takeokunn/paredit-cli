use std::fs;

use anyhow::{Context, Result};

use crate::application::usecase::definition_report::DefinitionReportItem;
use crate::domain::definition::definition_shape;
use crate::domain::sexpr::{Edit, SyntaxTree};

use super::super::shared::{
    detect_dialect, list_head, package_context_before_top_level, read_file_or_empty, read_input,
    write_files_with_rollback,
};
use super::args::MoveDefinitionArgs;
use super::render::move_definition::print_move_definition_plan;
use super::shared::append_top_level_form;
use super::types::MoveDefinitionPlan;

pub(in crate::presentation::cli) fn move_definition(args: MoveDefinitionArgs) -> Result<()> {
    let same_file = match (
        fs::canonicalize(&args.from_file),
        fs::canonicalize(&args.to_file),
    ) {
        (Ok(from), Ok(to)) => from == to,
        _ => args.from_file == args.to_file,
    };
    if same_file {
        anyhow::bail!("--from-file and --to-file must refer to different files");
    }

    let from_input = read_input(Some(args.from_file.clone()))?;
    let (to_input, to_file_existed) = read_file_or_empty(&args.to_file)?;
    let from_dialect = detect_dialect(&from_input, args.dialect);
    let to_dialect = detect_dialect(&to_input, args.dialect);
    let from_tree = SyntaxTree::parse(&from_input.text)?;
    SyntaxTree::parse(&to_input.text).with_context(|| {
        format!(
            "destination file is not a valid S-expression document: {}",
            args.to_file.display()
        )
    })?;

    let target_index = match args.path.indexes() {
        [index] => index.get(),
        _ => anyhow::bail!(
            "move-definition requires a top-level definition path, for example --path 2"
        ),
    };
    if target_index >= from_tree.root_children().len() {
        anyhow::bail!("top-level path {} is out of range", args.path);
    }

    let selection = from_tree.select_path(&args.path)?;
    let view = selection.view();
    let span = selection.span();
    let Some(head) = list_head(&view) else {
        anyhow::bail!("selected top-level form is not a list definition");
    };
    let Some(shape) = definition_shape(from_dialect, &view, head) else {
        anyhow::bail!("selected top-level form is not recognized as a definition: {head}");
    };

    let definition_text = selection.text(&from_input.text).to_owned();
    let source_package = package_context_before_top_level(&from_tree, target_index)?;
    let definition = DefinitionReportItem {
        path: args.path.to_string(),
        span,
        head: head.to_owned(),
        name: shape.name(&view).map(ToOwned::to_owned),
        category: shape.category,
        parameter_count: shape.lambda_parameter_count(&view),
        body_form_count: Some(shape.body_form_count(&view)),
        package: source_package.clone(),
    };
    let from_rewritten = Edit::kill(&from_input.text, &from_tree, selection)?;
    let to_tree = SyntaxTree::parse(&to_input.text)?;
    let dest_package = package_context_before_top_level(&to_tree, to_tree.root_children().len())?;
    let appended = match &source_package {
        Some(package) if dest_package.as_deref() != Some(package.as_str()) => {
            format!("(in-package {package})\n\n{definition_text}")
        }
        _ => definition_text.clone(),
    };
    let to_rewritten = append_top_level_form(&to_input.text, &appended);

    SyntaxTree::parse(&from_rewritten).with_context(|| {
        format!(
            "source file would become invalid after moving definition: {}",
            args.from_file.display()
        )
    })?;
    SyntaxTree::parse(&to_rewritten).with_context(|| {
        format!(
            "destination file would become invalid after receiving definition: {}",
            args.to_file.display()
        )
    })?;

    let changed = from_rewritten != from_input.text || to_rewritten != to_input.text;
    let written = args.write && changed;
    if written {
        write_files_with_rollback([
            (args.from_file.clone(), from_rewritten.clone()),
            (args.to_file.clone(), to_rewritten.clone()),
        ])?;
    }

    let plan = MoveDefinitionPlan {
        from_file: args.from_file,
        to_file: args.to_file,
        from_dialect,
        to_dialect,
        path: args.path,
        span,
        definition,
        definition_text,
        from_rewritten,
        to_rewritten,
        to_file_existed,
        changed,
        written,
    };
    print_move_definition_plan(&plan, args.output)
}
