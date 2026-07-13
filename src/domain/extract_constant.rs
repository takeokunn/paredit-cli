use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::extract_shared::{TopLevelInsert, insert_top_level_form, replace_span};
use crate::domain::sexpr::{
    ByteSpan, ExpressionKind, ExpressionView, Path, ReaderPrefix, Selection, SymbolName, SyntaxTree,
};

#[derive(Debug, Clone)]
pub(crate) struct Request<'a> {
    pub input: &'a str,
    pub tree: &'a SyntaxTree,
    pub selection: Selection<'a>,
    pub path: Path,
    pub dialect: Dialect,
    pub name: SymbolName,
    pub insert: TopLevelInsert,
    pub anchor_path: Option<Path>,
}

#[derive(Debug, Clone)]
pub(crate) struct Plan {
    pub dialect: Dialect,
    pub path: Path,
    pub span_start: usize,
    pub span_end: usize,
    pub name: SymbolName,
    pub anchor_path: Option<Path>,
    pub anchor_span: Option<ByteSpan>,
    pub replacement: String,
    pub definition: String,
    pub rewritten: String,
    pub changed: bool,
}

pub(crate) fn plan(request: Request<'_>) -> Result<Plan> {
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
        "extract-constant",
    )?;
    SyntaxTree::parse(&rewritten)
        .context("extracted output is not a valid S-expression document")?;

    Ok(Plan {
        dialect: request.dialect,
        path: request.path,
        span_start: span.start().get(),
        span_end: span.end().get(),
        name: request.name,
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
