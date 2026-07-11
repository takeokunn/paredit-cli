//! Use-case helpers for extracting one expression into a top-level constant.

use anyhow::{Context, Result};

use crate::application::usecase::extract_shared::{
    TopLevelInsert, insert_top_level_form, replace_span,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, ExpressionKind, ExpressionView, Path, ReaderPrefix, Selection, SymbolName, SyntaxTree,
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
    validate_dialect(request.dialect)?;
    validate_target(request.tree, &request.path, request.dialect)?;

    let span = request.selection.span();
    let selected = request.selection.text(request.input).to_owned();
    let replacement = request.name.as_str().to_owned();
    let head = match request.dialect {
        Dialect::CommonLisp => "defconstant",
        Dialect::EmacsLisp => "defconst",
        _ => unreachable!("validated dialect"),
    };
    let definition = format!("({head} {} {selected})", request.name);
    let replaced = replace_span(request.input, span, &replacement);
    let replaced_tree = SyntaxTree::parse(&replaced)
        .context("replacement output is not a valid S-expression document")?;
    let (rewritten, anchor_span) = insert_top_level_form(
        &replaced,
        &replaced_tree,
        &definition,
        request.insert,
        request.anchor_path.as_ref(),
        "extract-constant --anchor-path",
    )?;
    SyntaxTree::parse(&rewritten)
        .context("extracted output is not a valid S-expression document")?;

    Ok(ExtractConstantPlan {
        dialect: request.dialect,
        path: request.path,
        span_start: span.start().get(),
        span_end: span.end().get(),
        name: request.name,
        insert: request.insert,
        anchor_path: request.anchor_path,
        anchor_span,
        replacement,
        definition,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn validate_dialect(dialect: Dialect) -> Result<()> {
    if matches!(dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        Ok(())
    } else {
        anyhow::bail!("extract-constant supports only common-lisp and emacs-lisp")
    }
}

fn validate_target(tree: &SyntaxTree, path: &Path, dialect: Dialect) -> Result<()> {
    if path.indexes().len() < 2 {
        anyhow::bail!("extract-constant cannot select an entire top-level form");
    }

    let indexes = path.to_raw_indexes();
    let root = tree.root_view();
    let mut current = &root;
    for (depth, index) in indexes.iter().copied().enumerate() {
        reject_quoted_context(current)?;
        current = current
            .children
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("path {path} is out of range"))?;
        if depth + 1 < indexes.len() {
            reject_quoted_context(current)?;
        }
    }
    reject_quoted_context(current)?;

    let parent_path = path.parent().expect("nested path has a parent");
    let parent = tree.select_path(&parent_path)?.view();
    let selected_index = path.indexes().last().expect("non-empty path").get();
    if selected_index == 0
        && list_head(&parent).is_some_and(|head| dialect.is_definition_head(head))
    {
        anyhow::bail!("extract-constant cannot select a definition head");
    }
    Ok(())
}

fn reject_quoted_context(view: &ExpressionView) -> Result<()> {
    if view
        .reader_prefixes
        .iter()
        .any(|prefix| matches!(prefix, ReaderPrefix::Quote | ReaderPrefix::Quasiquote))
        || list_head(view).is_some_and(is_quote_head)
    {
        anyhow::bail!("extract-constant cannot select inside quote or quasiquote");
    }
    Ok(())
}

fn list_head(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::List)
        .then(|| view.children.first())
        .flatten()
        .filter(|head| head.kind == ExpressionKind::Atom && head.reader_prefixes.is_empty())
        .and_then(|head| head.text.as_deref())
}

fn is_quote_head(head: &str) -> bool {
    let normalized = head.rsplit(':').next().unwrap_or(head);
    normalized.eq_ignore_ascii_case("quote") || normalized.eq_ignore_ascii_case("quasiquote")
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
