use std::path::Path as FsPath;

use anyhow::Result;

use crate::application::form_similarity::StructuralTree;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path, SyntaxTree};

use super::types::{
    SimilarityCandidate, SimilarityFormReport, SimilarityFormScope, SimilarityReportOptions,
};

pub fn collect_similarity_candidates(
    tree: &SyntaxTree,
    input: &str,
    file: &FsPath,
    dialect: Dialect,
    options: &SimilarityReportOptions,
    candidates: &mut Vec<SimilarityCandidate>,
) -> Result<()> {
    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collect_from_view(&view, input, file, dialect, path, options, candidates);
    }
    Ok(())
}

fn collect_from_view(
    view: &ExpressionView,
    input: &str,
    file: &FsPath,
    dialect: Dialect,
    path: Path,
    options: &SimilarityReportOptions,
    candidates: &mut Vec<SimilarityCandidate>,
) {
    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        let tree = StructuralTree::from_view(view);
        let line_span = view
            .span
            .slice(input)
            .bytes()
            .filter(|&byte| byte == b'\n')
            .count()
            + 1;
        let in_scope = options.form_scope == SimilarityFormScope::All || path.indexes().len() == 1;
        if in_scope
            && tree.node_count() >= options.min_node_count
            && line_span >= options.min_line_span
        {
            candidates.push(SimilarityCandidate {
                form: SimilarityFormReport {
                    path: file.to_path_buf(),
                    dialect,
                    form_path: path.to_string(),
                    span: view.span,
                    node_count: tree.node_count(),
                    head: view
                        .children
                        .first()
                        .and_then(atom_text)
                        .map(ToOwned::to_owned),
                    text: view.span.slice(input).to_owned(),
                },
                tree,
            });
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_from_view(
            child,
            input,
            file,
            dialect,
            path.child(index),
            options,
            candidates,
        );
    }
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
