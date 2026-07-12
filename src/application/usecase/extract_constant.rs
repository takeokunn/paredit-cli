//! Use-case helpers for extracting one expression into a top-level constant.

use anyhow::Result;

use crate::application::usecase::extract_shared::TopLevelInsert;
use crate::application::usecase::mutation_safety::reject_overlapping_common_lisp_reader_time_forms;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, ExpressionKind, ExpressionView, Path, Selection, SymbolName, SyntaxTree,
};

pub type ExtractConstantInsert = TopLevelInsert;

#[derive(Debug, Clone)]
pub struct ExtractConstantRequest<'a> {
    pub input: &'a str,
    pub tree: &'a SyntaxTree,
    pub selection: Selection<'a>,
    pub path: Path,
    pub dialect: Dialect,
    pub name: SymbolName,
    pub insert: ExtractConstantInsert,
    pub anchor_path: Option<Path>,
}

#[derive(Debug, Clone)]
pub struct ExtractConstantPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub span_start: usize,
    pub span_end: usize,
    pub name: SymbolName,
    pub insert: ExtractConstantInsert,
    pub anchor_path: Option<Path>,
    pub anchor_span: Option<ByteSpan>,
    pub replacement: String,
    pub definition: String,
    pub rewritten: String,
    pub changed: bool,
}

pub fn path_for_selection(tree: &SyntaxTree, selection: Selection<'_>) -> Result<Path> {
    let target = selection.span();
    find_path(&tree.root_view(), target, &mut Vec::new())
        .map(Path::from_indexes)
        .ok_or_else(|| anyhow::anyhow!("selected expression path could not be resolved"))
}

pub fn plan_extract_constant(request: ExtractConstantRequest<'_>) -> Result<ExtractConstantPlan> {
    if request.dialect == Dialect::CommonLisp {
        reject_overlapping_common_lisp_reader_time_forms(
            request.tree,
            request.dialect,
            [request.selection.span()],
        )?;
    }
    let plan = crate::domain::extract_constant::plan(crate::domain::extract_constant::Request {
        input: request.input,
        tree: request.tree,
        selection: request.selection,
        path: request.path,
        dialect: request.dialect,
        name: request.name,
        insert: request.insert,
        anchor_path: request.anchor_path,
    })?;
    Ok(ExtractConstantPlan {
        dialect: plan.dialect,
        path: plan.path,
        span_start: plan.span_start,
        span_end: plan.span_end,
        name: plan.name,
        insert: request.insert,
        anchor_path: plan.anchor_path,
        anchor_span: plan.anchor_span,
        replacement: plan.replacement,
        definition: plan.definition,
        changed: plan.changed,
        rewritten: plan.rewritten,
    })
}

fn find_path(view: &ExpressionView, target: ByteSpan, path: &mut Vec<usize>) -> Option<Vec<usize>> {
    if view.kind != ExpressionKind::Root && view.span == target {
        return Some(path.clone());
    }
    for (index, child) in view.children.iter().enumerate() {
        path.push(index);
        if let Some(found) = find_path(child, target, path) {
            return Some(found);
        }
        path.pop();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn plan(input: &str, path: &str, dialect: Dialect) -> Result<ExtractConstantPlan> {
        let tree = SyntaxTree::parse(input)?;
        let path = Path::from_str(path)?;
        let selection = tree.select_path(&path)?;
        plan_extract_constant(ExtractConstantRequest {
            input,
            tree: &tree,
            selection,
            path,
            dialect,
            name: SymbolName::new("answer")?,
            insert: ExtractConstantInsert::Append,
            anchor_path: None,
        })
    }

    #[test]
    fn plans_common_lisp_constant() {
        let plan = plan("(defun f () (+ 40 2))", "0.3", Dialect::CommonLisp).unwrap();
        assert_eq!(plan.definition, "(defconstant answer (+ 40 2))");
        assert_eq!(
            plan.rewritten,
            "(defun f () answer)\n\n(defconstant answer (+ 40 2))\n"
        );
    }

    #[test]
    fn plans_emacs_lisp_constant() {
        let plan = plan("(defun f () (+ 40 2))", "0.3", Dialect::EmacsLisp).unwrap();
        assert_eq!(plan.definition, "(defconst answer (+ 40 2))");
    }

    #[test]
    fn rejects_quote_and_definition_head() {
        assert!(
            plan("(defun f () '(+ 40 2))", "0.3.1", Dialect::CommonLisp)
                .unwrap_err()
                .to_string()
                .contains("quote")
        );
        assert!(
            plan("(defun f () 42)", "0.0", Dialect::CommonLisp)
                .unwrap_err()
                .to_string()
                .contains("definition head")
        );
    }
}
