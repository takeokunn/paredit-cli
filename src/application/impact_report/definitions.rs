use anyhow::Result;

use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SymbolName, SyntaxTree};

use super::syntax::{
    atom_child, body_form_count, count_lambda_parameters, definition_name, lambda_list_index,
    list_head,
};
use super::types::ImpactDefinitionItem;

pub(super) fn collect_impact_definitions(
    tree: &SyntaxTree,
    dialect: Dialect,
) -> Result<(Option<String>, Vec<ImpactDefinitionItem>)> {
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

        definitions.push(ImpactDefinitionItem {
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

pub(super) fn impact_definition_matches_signature(
    definition: &ImpactDefinitionItem,
    symbol: Option<&SymbolName>,
) -> bool {
    definition.parameter_count.is_some()
        && definition.category.is_callable()
        && definition
            .name
            .as_deref()
            .is_some_and(|name| symbol.is_none_or(|target| name == target.as_str()))
}
