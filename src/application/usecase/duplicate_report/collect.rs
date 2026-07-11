use std::path::Path as FsPath;

use anyhow::Result;

use crate::application::form_shape::duplicate_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path, SyntaxTree};

use super::syntax::{atom_text, expression_node_count};
use super::types::{DuplicateCandidateGroups, DuplicateFormReport};

pub fn collect_duplicate_candidates(
    tree: &SyntaxTree,
    input: &str,
    file: &FsPath,
    dialect: Dialect,
    min_node_count: usize,
    grouped: &mut DuplicateCandidateGroups,
) -> Result<()> {
    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collect_duplicate_candidates_from_view(
            &view,
            input,
            file,
            dialect,
            path,
            min_node_count,
            grouped,
        );
    }

    Ok(())
}

fn collect_duplicate_candidates_from_view(
    view: &ExpressionView,
    input: &str,
    file: &FsPath,
    dialect: Dialect,
    path: Path,
    min_node_count: usize,
    grouped: &mut DuplicateCandidateGroups,
) {
    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        let node_count = expression_node_count(view);
        if node_count >= min_node_count {
            let shape = duplicate_shape(view, true);
            grouped.entry(shape).or_default().push(DuplicateFormReport {
                path: file.to_path_buf(),
                dialect,
                form_path: path.to_string(),
                span: view.span,
                node_count,
                head: view
                    .children
                    .first()
                    .and_then(atom_text)
                    .map(ToOwned::to_owned),
                text: view.span.slice(input).to_owned(),
            });
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        let child_path = path.child(index);
        collect_duplicate_candidates_from_view(
            child,
            input,
            file,
            dialect,
            child_path,
            min_node_count,
            grouped,
        );
    }
}
