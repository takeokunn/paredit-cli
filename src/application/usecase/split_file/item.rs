use anyhow::Result;

use crate::domain::common_lisp::CommonLispPackageDeclarationForm;
use crate::domain::definition::definition_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SyntaxTree};

use super::syntax::{atom_child, list_head};
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
    let Some(shape) = definition_shape(from_dialect, &view, head) else {
        anyhow::bail!(
            "selected top-level form is not recognized as a definition at {path}: {head}"
        );
    };

    let definition_text = selection.text(from_input).to_owned();
    let definition = SplitFileDefinition {
        path: path.to_string(),
        span,
        head: head.to_owned(),
        name: shape.name(&view).map(ToOwned::to_owned),
        category: shape.category,
        parameter_count: shape.lambda_parameter_count(&view),
        body_form_count: Some(shape.body_form_count(&view)),
        package: package_context_before_top_level(from_tree, from_dialect, target_index)?,
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
    dialect: Dialect,
    target_index: usize,
) -> Result<Option<String>> {
    let mut current_package = None;
    for index in 0..target_index {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        if list_head(&view)
            .and_then(|head| dialect.common_lisp_package_declaration_form_for_head(head))
            == Some(CommonLispPackageDeclarationForm::InPackage)
        {
            if let Some(package_name) = atom_child(&view, 1) {
                current_package = Some(package_name.to_owned());
            }
        }
    }
    Ok(current_package)
}
