//! Use case for converting a selected `cond` form into nested `if` forms.

use anyhow::{bail, Context, Result};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct ConvertCondToIfRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct ConvertCondToIfPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub clause_count: usize,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_convert_cond_to_if(request: ConvertCondToIfRequest<'_>) -> Result<ConvertCondToIfPlan> {
    if !matches!(request.dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("convert-cond-to-if currently supports only Common Lisp and Emacs Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("convert-cond-to-if input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    if tree.has_comment_in(form.span) {
        bail!("convert-cond-to-if cannot rewrite a form containing comments");
    }
    require_cond_form(&form, request.dialect)?;
    let clauses = &form.children[1..];
    if clauses.is_empty() {
        bail!("convert-cond-to-if requires at least one clause");
    }
    for clause in clauses {
        require_simple_clause(clause)?;
    }

    let mut replacement = None;
    for clause in clauses.iter().rev() {
        let test = clause.children[0].span.slice(request.input);
        let consequent = clause.children[1].span.slice(request.input);
        replacement = Some(match replacement {
            Some(else_form) => format!("(if {test} {consequent} {else_form})"),
            None => format!("(if {test} {consequent})"),
        });
    }
    let replacement = replacement.expect("non-empty clauses checked above");
    let rewritten = replace_span(request.input, form.span, &replacement);
    SyntaxTree::parse(&rewritten)
        .context("convert-cond-to-if output is not a valid S-expression document")?;

    Ok(ConvertCondToIfPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: form.span,
        clause_count: clauses.len(),
        changed: rewritten != request.input,
        rewritten,
    })
}

fn require_cond_form(form: &ExpressionView, dialect: Dialect) -> Result<()> {
    if form.kind != ExpressionKind::List || !form.reader_prefixes.is_empty() {
        bail!("convert-cond-to-if selected form must be a plain cond form");
    }
    let matches = form
        .children
        .first()
        .filter(|head| head.reader_prefixes.is_empty())
        .and_then(atom_symbol_text)
        .is_some_and(|head| match dialect {
            Dialect::CommonLisp => common_lisp_symbol_reference_eq(head, "cond"),
            Dialect::EmacsLisp => head == "cond",
            _ => false,
        });
    if !matches {
        bail!("convert-cond-to-if selected form must be a cond form");
    }
    Ok(())
}

fn require_simple_clause(clause: &ExpressionView) -> Result<()> {
    if clause.kind != ExpressionKind::List
        || !clause.reader_prefixes.is_empty()
        || clause.children.len() != 2
    {
        bail!("convert-cond-to-if requires each clause to contain exactly test and consequent");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(input: &str, dialect: Dialect) -> ConvertCondToIfRequest<'_> {
        ConvertCondToIfRequest {
            input,
            dialect,
            path: "0".parse().expect("path"),
        }
    }

    #[test]
    fn converts_common_lisp_clauses_to_nested_if_and_preserves_expressions() {
        let input = "(cond ((ready-p item) (use item))\n      ((waiting-p item) (wait-for item)))";
        let plan = plan_convert_cond_to_if(request(input, Dialect::CommonLisp)).expect("plan");
        assert_eq!(
            plan.rewritten,
            "(if (ready-p item) (use item) (if (waiting-p item) (wait-for item)))"
        );
        assert_eq!(plan.clause_count, 2);
    }

    #[test]
    fn converts_single_emacs_lisp_clause_without_else() {
        let plan = plan_convert_cond_to_if(request(
            "(cond (ready (message \"ok\")))",
            Dialect::EmacsLisp,
        ))
        .expect("plan");
        assert_eq!(plan.rewritten, "(if ready (message \"ok\"))");
    }

    #[test]
    fn rejects_empty_test_only_multi_body_and_non_list_clauses() {
        assert!(plan_convert_cond_to_if(request("(cond)", Dialect::CommonLisp)).is_err());
        assert!(plan_convert_cond_to_if(request("(cond (test))", Dialect::CommonLisp)).is_err());
        assert!(
            plan_convert_cond_to_if(request("(cond (test one two))", Dialect::EmacsLisp)).is_err()
        );
        assert!(plan_convert_cond_to_if(request("(cond test)", Dialect::EmacsLisp)).is_err());
    }

    #[test]
    fn rejects_comments_reader_conditionals_and_non_plain_cond() {
        assert!(plan_convert_cond_to_if(request(
            "(cond ; keep\n (test body))",
            Dialect::CommonLisp
        ))
        .is_err());
        assert!(
            plan_convert_cond_to_if(request("(cond (#+sbcl test body))", Dialect::CommonLisp))
                .is_err()
        );
        assert!(
            plan_convert_cond_to_if(request("'(cond (test body))", Dialect::EmacsLisp)).is_err()
        );
        assert!(plan_convert_cond_to_if(request("(cond (test body))", Dialect::Clojure)).is_err());
    }
}
