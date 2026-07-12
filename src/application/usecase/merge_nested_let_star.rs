//! Merge a directly nested `let*` while preserving sequential binding semantics.

use anyhow::{Context, Result, bail};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct MergeNestedLetStarRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct MergeNestedLetStarPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub outer_binding_count: usize,
    pub inner_binding_count: usize,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_merge_nested_let_star(
    request: MergeNestedLetStarRequest<'_>,
) -> Result<MergeNestedLetStarPlan> {
    if !matches!(request.dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("merge-nested-let-star supports only Common Lisp and Emacs Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("merge-nested-let-star input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let outer = tree.select_path(&request.path)?.view();
    require_let_star(request.dialect, &outer, "selected form")?;
    if tree.has_comment_in(outer.span) {
        bail!("merge-nested-let-star cannot rewrite a form containing comments");
    }
    if contains_reader_prefix(&outer) {
        bail!("merge-nested-let-star conservatively rejects reader prefixes");
    }
    if contains_headed_form(request.dialect, &outer, "declare") {
        bail!("merge-nested-let-star conservatively rejects declarations");
    }
    if outer.children.len() != 3 {
        bail!("merge-nested-let-star requires the outer body to contain only one form");
    }

    let outer_bindings = require_binding_list(&outer.children[1], "outer")?;
    let inner = &outer.children[2];
    require_let_star(request.dialect, inner, "outer body")?;
    if inner.children.len() < 3 {
        bail!("merge-nested-let-star requires the inner let* to have a body");
    }
    let inner_bindings = require_binding_list(&inner.children[1], "inner")?;

    let outer_text = list_contents(request.input, outer_bindings);
    let inner_text = list_contents(request.input, inner_bindings);
    let separator = if outer_text.trim().is_empty() || inner_text.trim().is_empty() {
        ""
    } else {
        " "
    };
    let head =
        &request.input[outer.children[0].span.start().get()..outer.children[0].span.end().get()];
    let body = &request.input[inner_bindings.span.end().get()..inner.span.end().get() - 1];
    let replacement = format!("({head} ({outer_text}{separator}{inner_text}){body})");
    let rewritten = replace_span(request.input, outer.span, &replacement);
    SyntaxTree::parse(&rewritten)
        .context("merge-nested-let-star output is not a valid S-expression document")?;

    Ok(MergeNestedLetStarPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: outer.span,
        outer_binding_count: outer_bindings.children.len(),
        inner_binding_count: inner_bindings.children.len(),
        changed: rewritten != request.input,
        rewritten,
    })
}

fn require_let_star(dialect: Dialect, view: &ExpressionView, role: &str) -> Result<()> {
    if view.kind != ExpressionKind::List
        || !view.reader_prefixes.is_empty()
        || !view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| {
                if dialect == Dialect::CommonLisp {
                    common_lisp_symbol_reference_eq(head, "let*")
                } else {
                    head == "let*"
                }
            })
    {
        bail!("merge-nested-let-star {role} must be a plain let* form");
    }
    Ok(())
}

fn require_binding_list<'a>(view: &'a ExpressionView, role: &str) -> Result<&'a ExpressionView> {
    if view.kind != ExpressionKind::List || !view.reader_prefixes.is_empty() {
        bail!("merge-nested-let-star requires a plain {role} binding list");
    }
    Ok(view)
}

fn list_contents<'a>(input: &'a str, view: &ExpressionView) -> &'a str {
    &input[view.span.start().get() + 1..view.span.end().get() - 1]
}

fn contains_reader_prefix(view: &ExpressionView) -> bool {
    !view.reader_prefixes.is_empty() || view.children.iter().any(contains_reader_prefix)
}

fn contains_headed_form(dialect: Dialect, view: &ExpressionView, expected: &str) -> bool {
    (view.kind == ExpressionKind::List
        && view.reader_prefixes.is_empty()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn request(input: &str, dialect: Dialect) -> MergeNestedLetStarRequest<'_> {
        MergeNestedLetStarRequest {
            input,
            dialect,
            path: "0".parse().expect("path"),
        }
    }

    #[test]
    fn merges_both_dialects_and_preserves_inner_body() {
        for dialect in [Dialect::CommonLisp, Dialect::EmacsLisp] {
            let input = "(let* ((x 1)) (let* ((y (+ x 1)))\n  (+ x y)))";
            let plan = plan_merge_nested_let_star(request(input, dialect)).expect("plan");
            assert_eq!(plan.rewritten, "(let* ((x 1) (y (+ x 1)))\n  (+ x y))");
            assert_eq!(plan.outer_binding_count, 1);
            assert_eq!(plan.inner_binding_count, 1);
        }
    }

    #[test]
    fn accepts_empty_binding_lists() {
        let plan =
            plan_merge_nested_let_star(request("(let* () (let* ((x 1)) x))", Dialect::CommonLisp))
                .expect("plan");
        assert_eq!(plan.rewritten, "(let* ((x 1)) x)");
    }

    #[test]
    fn rejects_extra_outer_body_and_non_nested_form() {
        assert!(
            plan_merge_nested_let_star(request(
                "(let* ((x 1)) (print x) (let* ((y 2)) y))",
                Dialect::CommonLisp,
            ))
            .is_err()
        );
        assert!(
            plan_merge_nested_let_star(request("(let* ((x 1)) (+ x 1))", Dialect::EmacsLisp,))
                .is_err()
        );
    }

    #[test]
    fn rejects_comments_declarations_and_reader_syntax() {
        assert!(
            plan_merge_nested_let_star(request(
                "(let* ((x 1)) ; keep\n (let* ((y 2)) y))",
                Dialect::EmacsLisp,
            ))
            .is_err()
        );
        assert!(
            plan_merge_nested_let_star(request(
                "(let* ((x 1)) (let* ((y 2)) (declare (special y)) y))",
                Dialect::CommonLisp,
            ))
            .is_err()
        );
        assert!(
            plan_merge_nested_let_star(request(
                "(let* ((x 'value)) (let* ((y 2)) y))",
                Dialect::CommonLisp,
            ))
            .is_err()
        );
        assert!(
            plan_merge_nested_let_star(request(
                "(let* ((x #+sbcl 1)) (let* ((y 2)) y))",
                Dialect::CommonLisp,
            ))
            .is_err()
        );
    }
}
