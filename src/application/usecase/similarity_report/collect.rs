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
) -> Result<usize> {
    let mut collection = CandidateCollection {
        input,
        file,
        dialect,
        options,
        candidates,
        omitted_candidates: 0,
    };
    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collection.collect_from_view(&view, path);
    }
    Ok(collection.omitted_candidates)
}

struct CandidateCollection<'a> {
    input: &'a str,
    file: &'a FsPath,
    dialect: Dialect,
    options: &'a SimilarityReportOptions,
    candidates: &'a mut Vec<SimilarityCandidate>,
    omitted_candidates: usize,
}

impl CandidateCollection<'_> {
    fn collect_from_view(&mut self, view: &ExpressionView, path: Path) {
        if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
            let tree = StructuralTree::from_view(view);
            let line_span = view
                .span
                .slice(self.input)
                .bytes()
                .filter(|&byte| byte == b'\n')
                .count()
                + 1;
            let in_scope =
                self.options.form_scope == SimilarityFormScope::All || path.indexes().len() == 1;
            if in_scope
                && tree.node_count() >= self.options.min_node_count
                && line_span >= self.options.min_line_span
            {
                if self
                    .options
                    .max_candidates
                    .is_some_and(|limit| self.candidates.len() >= limit)
                {
                    self.omitted_candidates = self.omitted_candidates.saturating_add(1);
                } else {
                    self.candidates.push(SimilarityCandidate {
                        form: SimilarityFormReport {
                            path: self.file.to_path_buf(),
                            dialect: self.dialect,
                            form_path: path.to_string(),
                            span: view.span,
                            node_count: tree.node_count(),
                            head: view
                                .children
                                .first()
                                .and_then(atom_text)
                                .map(ToOwned::to_owned),
                            text: view.span.slice(self.input).to_owned(),
                        },
                        tree,
                    });
                }
            }
        }

        for (index, child) in view.children.iter().enumerate() {
            self.collect_from_view(child, path.child(index));
        }
    }
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
