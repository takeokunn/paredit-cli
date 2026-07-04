use anyhow::Result;

use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path, SyntaxTree};

use super::types::DependencyReportItem;

mod asdf;
mod forms;
mod qualified;

pub(super) fn collect_dependency_items(tree: &SyntaxTree) -> Result<Vec<DependencyReportItem>> {
    let mut dependencies = Vec::new();

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_dependency_items_from_view(&view, path_indexes, &mut dependencies);
    }

    Ok(dependencies)
}

fn collect_dependency_items_from_view(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    dependencies: &mut Vec<DependencyReportItem>,
) {
    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        forms::collect_list_dependency_items(view, &path_indexes, dependencies);
        asdf::collect_system_dependency_items(view, &path_indexes, dependencies);
    }

    qualified::collect_qualified_symbol_dependency(view, &path_indexes, dependencies);

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_dependency_items_from_view(child, child_path, dependencies);
    }
}
