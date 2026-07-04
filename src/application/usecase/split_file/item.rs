use anyhow::Result;

use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SyntaxTree};

use super::syntax::{
    atom_child, body_form_count, count_lambda_parameters, definition_name, lambda_list_index,
    list_head,
};
use super::{SplitFileDefinition, SplitFileItem};

pub(super) fn build_split_file_item(
    from_tree: &SyntaxTree,
    from_input: &str,
    from_dialect: Dialect,
    path: Path,
    target_index: usize,
) -> Result<SplitFileItem> {
    let selection = from_tree.select_path(&path)?;
    let view = selection.view();
    let span = selection.span();
    let Some(head) = list_head(&view) else {
        anyhow::bail!("selected top-level form is not a list definition: {path}");
    };
    let Some(category) = classify_definition_head(from_dialect, head) else {
        anyhow::bail!(
            "selected top-level form is not recognized as a definition at {path}: {head}"
        );
    };

    let definition_text = selection.text(from_input).to_owned();
    let lambda_index = lambda_list_index(&view, head);
    let definition = SplitFileDefinition {
        path: path.to_string(),
        span,
        head: head.to_owned(),
        name: definition_name(&view, head).map(ToOwned::to_owned),
        category,
        parameter_count: lambda_index.map(|index| count_lambda_parameters(&view.children[index])),
        body_form_count: body_form_count(&view, lambda_index),
        package: package_context_before_top_level(from_tree, target_index)?,
    };

    Ok(SplitFileItem {
        path,
        span,
        removal_span: span,
        definition,
        definition_text,
    })
}

fn package_context_before_top_level(
    tree: &SyntaxTree,
    target_index: usize,
) -> Result<Option<String>> {
    let mut current_package = None;
    for index in 0..target_index {
        let path = Path::from_indexes(vec![index]);
        let view = tree.select_path(&path)?.view();
        if list_head(&view).is_some_and(|head| head.eq_ignore_ascii_case("in-package")) {
            if let Some(package_name) = atom_child(&view, 1) {
                current_package = Some(package_name.to_owned());
            }
        }
    }
    Ok(current_package)
}
