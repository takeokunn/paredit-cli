use anyhow::Result;

use crate::domain::definition::definition_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionView, Path, SyntaxTree};

use super::syntax::list_head;
use super::types::CallGraphDefinitionItem;

pub(super) fn collect_call_graph_definitions(
    tree: &SyntaxTree,
    dialect: Dialect,
) -> Result<Vec<CallGraphDefinitionItem>> {
    let mut items = Vec::new();

    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collect_call_graph_definitions_from_view(&view, dialect, path, &mut items);
    }

    Ok(items)
}

fn collect_call_graph_definitions_from_view(
    view: &ExpressionView,
    dialect: Dialect,
    path: Path,
    items: &mut Vec<CallGraphDefinitionItem>,
) {
    if view.kind == crate::domain::sexpr::ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
    {
        if let Some(head) = list_head(view) {
            if let Some(shape) = definition_shape(dialect, view, head) {
                let name = shape.name(view).map(str::to_string);
                let parameter_count = shape.lambda_parameter_count(view).unwrap_or(0);

                items.push(CallGraphDefinitionItem {
                    name,
                    category: shape.category,
                    path: path.clone(),
                    span: view.span,
                    parameter_count,
                });
            }
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        let child_path = path.child(index);
        collect_call_graph_definitions_from_view(child, dialect, child_path, items);
    }
}
