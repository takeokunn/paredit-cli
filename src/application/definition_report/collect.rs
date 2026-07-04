use std::path::PathBuf;

use anyhow::Result;

use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SyntaxTree};

use super::syntax::{
    atom_child, body_form_count, count_lambda_parameters, definition_name, lambda_list_index,
    list_head,
};
use super::types::{DefinitionReportFile, DefinitionReportItem, ParsedDefinitionFile};

pub fn build_definition_report(
    path: PathBuf,
    dialect: Dialect,
    tree: &SyntaxTree,
) -> Result<DefinitionReportFile> {
    let (package, definitions) = collect_definition_forms(tree, dialect)?;

    Ok(DefinitionReportFile {
        path,
        dialect,
        package,
        definitions,
    })
}

pub fn build_parsed_definition_file(
    path: PathBuf,
    dialect: Dialect,
    tree: &SyntaxTree,
) -> Result<ParsedDefinitionFile> {
    let (package, definitions) = collect_definition_forms(tree, dialect)?;

    Ok(ParsedDefinitionFile {
        path,
        dialect,
        package,
        definitions,
        atoms: tree.atom_occurrences(),
    })
}

pub fn collect_definition_forms(
    tree: &SyntaxTree,
    dialect: Dialect,
) -> Result<(Option<String>, Vec<DefinitionReportItem>)> {
    let mut current_package = None;
    let mut definitions = Vec::new();

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        let Some(head) = list_head(&view) else {
            continue;
        };

        if head.eq_ignore_ascii_case("in-package") {
            if let Some(package_name) = atom_child(&view, 1) {
                current_package = Some(package_name.to_owned());
            }
            continue;
        }

        let Some(category) = classify_definition_head(dialect, head) else {
            continue;
        };
        let lambda_index = lambda_list_index(&view, head);

        definitions.push(DefinitionReportItem {
            path: Path::from_indexes(path_indexes).to_string(),
            span: view.span,
            head: head.to_owned(),
            name: definition_name(&view, head).map(ToOwned::to_owned),
            category,
            parameter_count: lambda_index
                .map(|index| count_lambda_parameters(&view.children[index])),
            body_form_count: body_form_count(&view, lambda_index),
            package: current_package.clone(),
        });
    }

    Ok((current_package, definitions))
}
