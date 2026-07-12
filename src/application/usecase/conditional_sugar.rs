//! Shared implementation for conversions between `if`, `when`, and `unless`.

use anyhow::{Context, Result, bail};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct ConditionalConversionRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}
#[derive(Debug, Clone)]
pub struct ConditionalConversionPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub body_count: usize,
    pub rewritten: String,
    pub changed: bool,
}

fn prepare<'a>(
    request: &ConditionalConversionRequest<'a>,
    head: &str,
) -> Result<(SyntaxTree, ExpressionView)> {
    if !matches!(request.dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("conditional conversion supports only Common Lisp and Emacs Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("conditional conversion input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    if tree.has_comment_in(form.span) {
        bail!("conditional conversion cannot rewrite a form containing comments");
    }
    if form.kind != ExpressionKind::List || !form.reader_prefixes.is_empty() {
        bail!("selected form must be a plain {head} form");
    }
    let matches = form
        .children
        .first()
        .filter(|v| v.reader_prefixes.is_empty())
        .and_then(atom_symbol_text)
        .is_some_and(|actual| match request.dialect {
            Dialect::CommonLisp => common_lisp_symbol_reference_eq(actual, head),
            Dialect::EmacsLisp => actual == head,
            _ => false,
        });
    if !matches {
        bail!("selected form must be a {head} form");
    }
    Ok((tree, form))
}

fn finish(
    request: ConditionalConversionRequest<'_>,
    form: &ExpressionView,
    body_count: usize,
    replacement: String,
) -> Result<ConditionalConversionPlan> {
    let rewritten = replace_span(request.input, form.span, &replacement);
    SyntaxTree::parse(&rewritten)
        .context("conditional conversion output is not a valid S-expression document")?;
    Ok(ConditionalConversionPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: form.span,
        body_count,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn literal_nil(view: &ExpressionView, dialect: Dialect) -> bool {
    view.kind == ExpressionKind::Atom
        && view.reader_prefixes.is_empty()
        && atom_symbol_text(view).is_some_and(|text| match dialect {
            Dialect::CommonLisp => common_lisp_symbol_reference_eq(text, "nil"),
            Dialect::EmacsLisp => text == "nil",
            _ => false,
        })
}

pub fn plan_convert_when_to_if(
    request: ConditionalConversionRequest<'_>,
) -> Result<ConditionalConversionPlan> {
    let (_tree, form) = prepare(&request, "when")?;
    if form.children.len() < 2 {
        bail!("convert-when-to-if requires a test");
    }
    let test = form.children[1].span.slice(request.input);
    let body = form.children[2..]
        .iter()
        .map(|v| v.span.slice(request.input))
        .collect::<Vec<_>>()
        .join(" ");
    finish(
        request,
        &form,
        form.children.len() - 2,
        format!(
            "(if {test} (progn{space}{body}))",
            space = if body.is_empty() { "" } else { " " }
        ),
    )
}

pub fn plan_convert_unless_to_if(
    request: ConditionalConversionRequest<'_>,
) -> Result<ConditionalConversionPlan> {
    let (_tree, form) = prepare(&request, "unless")?;
    if form.children.len() < 2 {
        bail!("convert-unless-to-if requires a test");
    }
    let test = form.children[1].span.slice(request.input);
    let body = form.children[2..]
        .iter()
        .map(|v| v.span.slice(request.input))
        .collect::<Vec<_>>()
        .join(" ");
    finish(
        request,
        &form,
        form.children.len() - 2,
        format!(
            "(if {test} nil (progn{space}{body}))",
            space = if body.is_empty() { "" } else { " " }
        ),
    )
}

pub fn plan_convert_if_to_when(
    request: ConditionalConversionRequest<'_>,
) -> Result<ConditionalConversionPlan> {
    let (_tree, form) = prepare(&request, "if")?;
    if !(3..=4).contains(&form.children.len()) {
        bail!("convert-if-to-when requires (if test then [nil])");
    }
    if form.children.len() == 4 && !literal_nil(&form.children[3], request.dialect) {
        bail!("convert-if-to-when requires no else form or a literal nil else");
    }
    let test = form.children[1].span.slice(request.input);
    let then = form.children[2].span.slice(request.input);
    finish(request, &form, 1, format!("(when {test} {then})"))
}

pub fn plan_convert_if_to_unless(
    request: ConditionalConversionRequest<'_>,
) -> Result<ConditionalConversionPlan> {
    let (_tree, form) = prepare(&request, "if")?;
    if form.children.len() != 4 || !literal_nil(&form.children[2], request.dialect) {
        bail!("convert-if-to-unless requires (if test nil else)");
    }
    let test = form.children[1].span.slice(request.input);
    let otherwise = form.children[3].span.slice(request.input);
    finish(request, &form, 1, format!("(unless {test} {otherwise})"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn req(input: &str, dialect: Dialect) -> ConditionalConversionRequest<'_> {
        ConditionalConversionRequest {
            input,
            dialect,
            path: "0".parse().unwrap(),
        }
    }
    #[test]
    fn converts_when_and_unless() {
        assert_eq!(
            plan_convert_when_to_if(req("(when ok one two)", Dialect::CommonLisp))
                .unwrap()
                .rewritten,
            "(if ok (progn one two))"
        );
        assert_eq!(
            plan_convert_unless_to_if(req("(unless ok one)", Dialect::EmacsLisp))
                .unwrap()
                .rewritten,
            "(if ok nil (progn one))"
        );
    }
    #[test]
    fn converts_if_to_when_and_unless() {
        assert_eq!(
            plan_convert_if_to_when(req("(if ok yes nil)", Dialect::CommonLisp))
                .unwrap()
                .rewritten,
            "(when ok yes)"
        );
        assert_eq!(
            plan_convert_if_to_unless(req("(if ok nil no)", Dialect::EmacsLisp))
                .unwrap()
                .rewritten,
            "(unless ok no)"
        );
    }
    #[test]
    fn rejects_invalid_shapes_comments_and_reader_conditionals() {
        assert!(plan_convert_when_to_if(req("(when)", Dialect::CommonLisp)).is_err());
        assert!(plan_convert_if_to_when(req("(if x y z)", Dialect::CommonLisp)).is_err());
        assert!(plan_convert_if_to_unless(req("(if x y z)", Dialect::EmacsLisp)).is_err());
        assert!(plan_convert_when_to_if(req("(when x ; c\n y)", Dialect::CommonLisp)).is_err());
        assert!(
            plan_convert_unless_to_if(req("(unless #+sbcl x y)", Dialect::CommonLisp)).is_err()
        );
        assert!(plan_convert_if_to_when(req("'(if x y)", Dialect::EmacsLisp)).is_err());
    }
}
