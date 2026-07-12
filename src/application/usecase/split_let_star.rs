//! Split a sequential `let*` binding list at an explicit boundary.

use anyhow::{Context, Result, bail};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct SplitLetStarRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
    pub binding_index: usize,
}

#[derive(Debug, Clone)]
pub struct SplitLetStarPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub binding_index: usize,
    pub outer_binding_count: usize,
    pub inner_binding_count: usize,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_split_let_star(request: SplitLetStarRequest<'_>) -> Result<SplitLetStarPlan> {
    if !matches!(request.dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("split-let-star supports only Common Lisp and Emacs Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("split-let-star input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    require_let_star(request.dialect, &form)?;
    if tree.has_comment_in(form.span) {
        bail!("split-let-star cannot rewrite a form containing comments");
    }
    if contains_reader_prefix(&form) {
        bail!("split-let-star conservatively rejects reader prefixes");
    }
    if contains_headed_form(request.dialect, &form, "declare") {
        bail!("split-let-star conservatively rejects declarations");
    }
    if form.children.len() < 3 {
        bail!("split-let-star requires a body");
    }
    let bindings = form
        .children
        .get(1)
        .context("split-let-star requires a binding list")?;
    if bindings.kind != ExpressionKind::List || !bindings.reader_prefixes.is_empty() {
        bail!("split-let-star requires a plain binding list");
    }
    if request.binding_index == 0 || request.binding_index >= bindings.children.len() {
        bail!(
            "split-let-star --binding-index must be between 1 and {}",
            bindings.children.len().saturating_sub(1)
        );
    }

    let head = form.children[0].span.slice(request.input);
    let outer = bindings.children[..request.binding_index]
        .iter()
        .map(|binding| binding.span.slice(request.input))
        .collect::<Vec<_>>()
        .join(" ");
    let inner = bindings.children[request.binding_index..]
        .iter()
        .map(|binding| binding.span.slice(request.input))
        .collect::<Vec<_>>()
        .join(" ");
    let body = &request.input[bindings.span.end().get()..form.span.end().get() - 1];
    let replacement = format!("({head} ({outer}) ({head} ({inner}){body}))");
    let rewritten = replace_span(request.input, form.span, &replacement);
    SyntaxTree::parse(&rewritten).context("split-let-star output is not valid")?;

    Ok(SplitLetStarPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: form.span,
        binding_index: request.binding_index,
        outer_binding_count: request.binding_index,
        inner_binding_count: bindings.children.len() - request.binding_index,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn require_let_star(dialect: Dialect, form: &ExpressionView) -> Result<()> {
    let matches = form.kind == ExpressionKind::List
        && form.reader_prefixes.is_empty()
        && form
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| {
                if dialect == Dialect::CommonLisp {
                    common_lisp_symbol_reference_eq(head, "let*")
                } else {
                    head == "let*"
                }
            });
    if !matches {
        bail!("split-let-star selected form must be a plain let* form");
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn request(input: &str, dialect: Dialect, binding_index: usize) -> SplitLetStarRequest<'_> {
        SplitLetStarRequest {
            input,
            dialect,
            path: "0".parse().expect("path"),
            binding_index,
        }
    }

    #[test]
    fn splits_at_boundary_in_both_dialects() {
        for dialect in [Dialect::CommonLisp, Dialect::EmacsLisp] {
            let plan = plan_split_let_star(request(
                "(let* ((a 1) (b (+ a 1)) (c (+ b 1))) (+ a b c))",
                dialect,
                1,
            ))
            .expect("plan");
            assert_eq!(
                plan.rewritten,
                "(let* ((a 1)) (let* ((b (+ a 1)) (c (+ b 1))) (+ a b c)))"
            );
        }
    }

    #[test]
    fn rejects_empty_sides_and_unsafe_syntax() {
        assert!(
            plan_split_let_star(request("(let* ((a 1) (b 2)) b)", Dialect::CommonLisp, 0)).is_err()
        );
        assert!(plan_split_let_star(request("(let* ((a 1)) a)", Dialect::CommonLisp, 1)).is_err());
        assert!(
            plan_split_let_star(request(
                "(let* ((a 1) (b 2)) (declare (special a)) b)",
                Dialect::CommonLisp,
                1
            ))
            .is_err()
        );
        assert!(
            plan_split_let_star(request("(let* ((a 'x) (b 2)) b)", Dialect::EmacsLisp, 1)).is_err()
        );
    }
}
