use std::path::Path as FsPath;

use anyhow::Result;

use crate::domain::common_lisp::normalize_common_lisp_operator_head;
use crate::domain::dialect::Dialect;
use crate::domain::form_similarity::StructuralTree;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SyntaxTree};

use super::types::{
    ComparisonHead, FormHead, SimilarityCandidate, SimilarityFormReport, SimilarityFormScope,
    SimilarityReportOptions,
};

pub fn collect_similarity_candidates(
    tree: &SyntaxTree,
    input: &str,
    file: &FsPath,
    dialect: Dialect,
    options: &SimilarityReportOptions,
    candidates: &mut Vec<SimilarityCandidate>,
) -> Result<usize> {
    options.validate()?;
    let line_index = (options.min_line_span() > 1).then_some(LineIndex::new(input));
    let mut collection = CandidateCollection {
        input,
        line_index: line_index.as_ref(),
        file,
        dialect,
        options,
        candidates,
        omitted_candidates: 0,
    };
    let mut path_stack = Vec::new();
    for index in 0..tree.root_children().len() {
        let view = tree.select_path(&Path::root_child(index))?.view();
        path_stack.push(index);
        collection.collect_from_view(&view, &mut path_stack);
        path_stack.pop();
    }
    Ok(collection.omitted_candidates)
}

struct CandidateCollection<'a> {
    input: &'a str,
    line_index: Option<&'a LineIndex>,
    file: &'a FsPath,
    dialect: Dialect,
    options: &'a SimilarityReportOptions,
    candidates: &'a mut Vec<SimilarityCandidate>,
    omitted_candidates: usize,
}

impl CandidateCollection<'_> {
    // `path_stack` is pushed/popped in place; a full `Path` is only built for
    // forms that actually become candidates, instead of cloning the whole
    // index vector at every recursion step (O(nodes × depth) allocation).
    fn collect_from_view(&mut self, view: &ExpressionView, path_stack: &mut Vec<usize>) {
        let is_top_level = path_stack.len() == 1;
        if self.options.form_scope() == SimilarityFormScope::TopLevel && !is_top_level {
            return;
        }
        if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
            let text = view.span.slice(self.input);
            let line_span = self
                .line_index
                .map_or(1, |line_index| line_index.line_span(view.span));
            let in_scope = self.options.form_scope() == SimilarityFormScope::All || is_top_level;
            if in_scope && line_span >= self.options.min_line_span() {
                if self
                    .options
                    .max_candidates()
                    .is_some_and(|limit| self.candidates.len() >= limit)
                {
                    self.omitted_candidates = self.omitted_candidates.saturating_add(1);
                } else {
                    let (tree, node_count) = StructuralTree::from_view_with_count(view);
                    if node_count >= self.options.min_node_count() {
                        let head = view.children.first().and_then(atom_text);
                        let form = SimilarityFormReport::new(
                            self.file.to_path_buf(),
                            self.dialect,
                            Path::from_indexes(path_stack.clone()),
                            view.span,
                            node_count,
                            head.map(FormHead::from),
                            text,
                        );
                        let comparison_head = head.map(|head| {
                            if self.dialect == Dialect::CommonLisp {
                                ComparisonHead::from(
                                    normalize_common_lisp_operator_head(head).to_ascii_lowercase(),
                                )
                            } else {
                                ComparisonHead::from(head.to_ascii_lowercase())
                            }
                        });
                        self.candidates
                            .push(SimilarityCandidate::new(form, tree, comparison_head));
                    }
                }
            }
        }

        if self.options.form_scope() == SimilarityFormScope::TopLevel && is_top_level {
            return;
        }

        for (index, child) in view.children.iter().enumerate() {
            path_stack.push(index);
            self.collect_from_view(child, path_stack);
            path_stack.pop();
        }
    }
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

#[derive(Debug)]
struct LineIndex {
    newline_offsets: Vec<usize>,
}

impl LineIndex {
    fn new(input: &str) -> Self {
        let newline_offsets = input
            .bytes()
            .enumerate()
            .filter_map(|(index, byte)| (byte == b'\n').then_some(index))
            .collect();
        Self { newline_offsets }
    }

    fn line_span(&self, span: ByteSpan) -> usize {
        let start = span.start().get();
        let end = span.end().get();
        let start_index = self
            .newline_offsets
            .partition_point(|&offset| offset < start);
        let end_index = self.newline_offsets.partition_point(|&offset| offset < end);
        end_index.saturating_sub(start_index) + 1
    }
}
