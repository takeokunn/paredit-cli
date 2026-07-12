//! Use case for converting a selected `if` form into `cond`.

use anyhow::{bail, Context, Result};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct ConvertIfToCondRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct ConvertIfToCondPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub has_else: bool,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_convert_if_to_cond(request: ConvertIfToCondRequest<'_>) -> Result<ConvertIfToCondPlan> {
    if !matches!(request.dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("convert-if-to-cond currently supports only Common Lisp and Emacs Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("convert-if-to-cond input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    if tree.has_comment_in(form.span) {
        bail!("convert-if-to-cond cannot rewrite a form containing comments");
    }
    require_if_form(&form, request.dialect)?;
    if !(3..=4).contains(&form.children.len()) {
        bail!("convert-if-to-cond requires (if test then [else])");
    }

    let test = form.children[1].span.slice(request.input);
    let then = form.children[2].span.slice(request.input);
    let replacement = match form.children.get(3) {
        Some(else_form) => format!(
            "(cond ({test} {then}) ((quote t) {}))",
            else_form.span.slice(request.input)
        ),
        None => format!("(cond ({test} {then}))"),
    };
    let rewritten = replace_span(request.input, form.span, &replacement);
    SyntaxTree::parse(&rewritten)
        .context("convert-if-to-cond output is not a valid S-expression document")?;

    Ok(ConvertIfToCondPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: form.span,
        has_else: form.children.len() == 4,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn require_if_form(form: &ExpressionView, dialect: Dialect) -> Result<()> {
    if form.kind != ExpressionKind::List || !form.reader_prefixes.is_empty() {
        bail!("convert-if-to-cond selected form must be a plain if form");
    }
    let matches = form
        .children
        .first()
        .filter(|head| head.reader_prefixes.is_empty())
        .and_then(atom_symbol_text)
        .is_some_and(|head| match dialect {
            Dialect::CommonLisp => common_lisp_symbol_reference_eq(head, "if"),
            Dialect::EmacsLisp => head == "if",
            _ => false,
        });
    if !matches {
        bail!("convert-if-to-cond selected form must be an if form");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(input: &str, dialect: Dialect) -> ConvertIfToCondRequest<'_> {
        ConvertIfToCondRequest {
            input,
            dialect,
            path: "0".parse().expect("path"),
        }
    }

    #[test]
    fn converts_common_lisp_if_with_else_and_preserves_expressions() {
        let input = "(if (ready-p item)\n    (use item)\n    (wait-for item))";
        let plan = plan_convert_if_to_cond(request(input, Dialect::CommonLisp)).expect("plan");
        assert_eq!(
            plan.rewritten,
            "(cond ((ready-p item) (use item)) ((quote t) (wait-for item)))"
        );
        assert!(plan.has_else);
    }

    #[test]
    fn converts_emacs_lisp_if_without_else() {
        let plan =
            plan_convert_if_to_cond(request("(if ready (message \"ok\"))", Dialect::EmacsLisp))
                .expect("plan");
        assert_eq!(plan.rewritten, "(cond (ready (message \"ok\")))");
        assert!(!plan.has_else);
    }

    #[test]
    fn uses_quoted_truth_for_emacs_lisp_else_clause() {
        let plan = plan_convert_if_to_cond(request("(if ready yes no)", Dialect::EmacsLisp))
            .expect("plan");
        assert_eq!(plan.rewritten, "(cond (ready yes) ((quote t) no))");
    }

    #[test]
    fn rejects_invalid_arity_comments_and_reader_conditionals() {
        assert!(plan_convert_if_to_cond(request("(if test)", Dialect::CommonLisp)).is_err());
        assert!(
            plan_convert_if_to_cond(request("(if test then else extra)", Dialect::EmacsLisp))
                .is_err()
        );
        assert!(
            plan_convert_if_to_cond(request("(if test ; keep\n then)", Dialect::CommonLisp))
                .is_err()
        );
        assert!(
            plan_convert_if_to_cond(request("(if #+sbcl test then)", Dialect::CommonLisp)).is_err()
        );
    }

    #[test]
    fn rejects_unsupported_dialect_and_non_plain_if() {
        assert!(plan_convert_if_to_cond(request("(if test then)", Dialect::Clojure)).is_err());
        assert!(plan_convert_if_to_cond(request("'(if test then)", Dialect::EmacsLisp)).is_err());
    }
}
