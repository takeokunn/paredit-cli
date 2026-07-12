//! Flatten a selected `progn` in a conservative expression context.

use anyhow::{Context, Result, bail};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct FlattenPrognRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct FlattenPrognPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub nested_count: usize,
    pub result_form_count: usize,
    pub replacement: String,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_flatten_progn(request: FlattenPrognRequest<'_>) -> Result<FlattenPrognPlan> {
    if !matches!(request.dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("flatten-progn supports only Common Lisp and Emacs Lisp");
    }
    if request.path.indexes().len() < 2 {
        bail!("flatten-progn refuses to rewrite a top-level progn");
    }

    let tree = SyntaxTree::parse(request.input)
        .context("flatten-progn input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    reject_unsafe_context(&tree, &request.path)?;

    let form = tree.select_path(&request.path)?.view();
    require_progn(request.dialect, &form, "selected form")?;
    if tree.has_comment_in(form.span) {
        bail!("flatten-progn cannot rewrite a form containing comments");
    }
    if contains_reader_prefix(&form) {
        bail!("flatten-progn conservatively rejects reader prefixes");
    }
    if contains_headed_form(request.dialect, &form, "declare") {
        bail!("flatten-progn conservatively rejects declarations");
    }

    let body = &form.children[1..];
    let nested_count = body
        .iter()
        .filter(|child| is_progn(request.dialect, child))
        .count();
    let mut flattened = Vec::new();
    for child in body {
        if is_progn(request.dialect, child) {
            flattened.extend(child.children[1..].iter());
        } else {
            flattened.push(child);
        }
    }

    let replacement = match flattened.as_slice() {
        [] => "nil".to_owned(),
        [only] => source_for(request.input, only).to_owned(),
        forms => {
            let head = source_for(request.input, &form.children[0]);
            let body = forms
                .iter()
                .map(|child| source_for(request.input, child))
                .collect::<Vec<_>>()
                .join(" ");
            format!("({head} {body})")
        }
    };
    let rewritten = replace_span(request.input, form.span, &replacement);
    SyntaxTree::parse(&rewritten)
        .context("flatten-progn output is not a valid S-expression document")?;

    Ok(FlattenPrognPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: form.span,
        nested_count,
        result_form_count: flattened.len(),
        changed: rewritten != request.input,
        replacement,
        rewritten,
    })
}

fn reject_unsafe_context(tree: &SyntaxTree, path: &Path) -> Result<()> {
    let indexes = path.indexes();
    if indexes.last().is_some_and(|index| index.get() == 0) {
        bail!("flatten-progn refuses to rewrite an operator position");
    }

    let mut ancestor = path.parent();
    while let Some(ancestor_path) = ancestor {
        if ancestor_path.indexes().is_empty() {
            break;
        }
        let view = tree.select_path(&ancestor_path)?.view();
        if !view.reader_prefixes.is_empty() {
            bail!("flatten-progn refuses to rewrite inside a reader template");
        }
        if view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| common_lisp_symbol_reference_eq(head, "declare"))
        {
            bail!("flatten-progn refuses to rewrite inside a declaration");
        }
        ancestor = ancestor_path.parent();
    }
    Ok(())
}

fn require_progn(dialect: Dialect, view: &ExpressionView, role: &str) -> Result<()> {
    if !is_progn(dialect, view) || !view.reader_prefixes.is_empty() {
        bail!("flatten-progn {role} must be a plain progn form");
    }
    Ok(())
}

fn is_progn(dialect: Dialect, view: &ExpressionView) -> bool {
    view.kind == ExpressionKind::List
        && view.reader_prefixes.is_empty()
        && view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| {
                if dialect == Dialect::CommonLisp {
                    common_lisp_symbol_reference_eq(head, "progn")
                } else {
                    head == "progn"
                }
            })
}

fn contains_reader_prefix(view: &ExpressionView) -> bool {
    !view.reader_prefixes.is_empty() || view.children.iter().any(contains_reader_prefix)
}

fn contains_headed_form(dialect: Dialect, view: &ExpressionView, expected: &str) -> bool {
    (view.kind == ExpressionKind::List
        && view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| {
                if dialect == Dialect::CommonLisp {
                    common_lisp_symbol_reference_eq(head, expected)
                } else {
                    head == expected
                }
            }))
        || view
            .children
            .iter()
            .any(|child| contains_headed_form(dialect, child, expected))
}

fn source_for<'a>(input: &'a str, view: &ExpressionView) -> &'a str {
    &input[view.span.start().get()..view.span.end().get()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request<'a>(input: &'a str, dialect: Dialect, path: &str) -> FlattenPrognRequest<'a> {
        FlattenPrognRequest {
            input,
            dialect,
            path: path.parse().expect("path"),
        }
    }

    #[test]
    fn flattens_direct_nested_progns_in_both_dialects() {
        for dialect in [Dialect::CommonLisp, Dialect::EmacsLisp] {
            let plan = plan_flatten_progn(request(
                "(print (progn one (progn two three) (progn four) five))",
                dialect,
                "0.1",
            ))
            .expect("plan");
            assert_eq!(plan.rewritten, "(print (progn one two three four five))");
            assert_eq!(plan.nested_count, 2);
            assert_eq!(plan.result_form_count, 5);
        }
    }

    #[test]
    fn removes_empty_and_singleton_progns() {
        let empty = plan_flatten_progn(request("(list (progn))", Dialect::CommonLisp, "0.1"))
            .expect("empty");
        assert_eq!(empty.rewritten, "(list nil)");

        let singleton =
            plan_flatten_progn(request("(list (progn value))", Dialect::EmacsLisp, "0.1"))
                .expect("singleton");
        assert_eq!(singleton.rewritten, "(list value)");
    }

    #[test]
    fn rejects_top_level_operator_declaration_and_reader_contexts() {
        for (input, path) in [
            ("(progn one two)", "0"),
            ("((progn one two) value)", "0.0"),
            ("(declare (custom (progn one two)))", "0.1.1"),
            ("`(list (progn one two))", "0.1"),
        ] {
            assert!(
                plan_flatten_progn(request(input, Dialect::CommonLisp, path)).is_err(),
                "input={input} path={path}"
            );
        }
    }

    #[test]
    fn rejects_comments_reader_prefixes_and_conditionals() {
        for input in [
            "(list (progn one ; keep\n two))",
            "(list (progn 'one two))",
            "(list (progn #+sbcl one two))",
        ] {
            assert!(
                plan_flatten_progn(request(input, Dialect::CommonLisp, "0.1")).is_err(),
                "input={input}"
            );
        }
    }
}
