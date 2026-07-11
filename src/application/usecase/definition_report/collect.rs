use std::path::PathBuf;

use anyhow::Result;

use crate::domain::common_lisp::CommonLispPackageDeclarationForm;
use crate::domain::definition::definition_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SyntaxTree};

use super::syntax::{atom_child, list_head};
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
    text: &str,
) -> Result<ParsedDefinitionFile> {
    let (package, definitions) = collect_definition_forms(tree, dialect)?;

    Ok(ParsedDefinitionFile {
        path,
        dialect,
        package,
        definitions,
        atoms: tree.atom_occurrences(),
        text: text.to_owned(),
    })
}

pub fn collect_definition_forms(
    tree: &SyntaxTree,
    dialect: Dialect,
) -> Result<(Option<String>, Vec<DefinitionReportItem>)> {
    let mut current_package = None;
    let mut definitions = Vec::new();

    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        let Some(head) = list_head(&view) else {
            continue;
        };

        if dialect.common_lisp_package_declaration_form_for_head(head)
            == Some(CommonLispPackageDeclarationForm::InPackage)
        {
            if let Some(package_name) = atom_child(&view, 1) {
                current_package = Some(package_name.to_owned());
            }
            continue;
        }

        let Some(shape) = definition_shape(dialect, &view, head) else {
            continue;
        };

        definitions.push(DefinitionReportItem {
            path: path.to_string(),
            span: view.span,
            head: head.to_owned(),
            name: shape.name(&view).map(ToOwned::to_owned),
            category: shape.category,
            parameter_count: shape.lambda_parameter_count(&view),
            body_form_count: Some(shape.body_form_count(&view)),
            package: current_package.clone(),
        });
    }

    Ok((current_package, definitions))
}
