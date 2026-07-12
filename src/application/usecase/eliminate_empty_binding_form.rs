//! Eliminate an empty `let` or `let*` in a known expression context.

use anyhow::{bail, Context, Result};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct EliminateEmptyBindingFormRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct EliminateEmptyBindingFormPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub body_form_count: usize,
    pub introduced_progn: bool,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_eliminate_empty_binding_form(
    request: EliminateEmptyBindingFormRequest<'_>,
) -> Result<EliminateEmptyBindingFormPlan> {
    if !matches!(request.dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("eliminate-empty-binding-form supports only Common Lisp and Emacs Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("eliminate-empty-binding-form input is not valid")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    require_empty_binding_form(request.dialect, &form)?;
    require_known_expression_context(&tree, &request.path, request.dialect)?;
    if tree.has_comment_in(form.span) {
        bail!("eliminate-empty-binding-form cannot rewrite a form containing comments");
    }
    if contains_reader_prefix(&form) {
        bail!("eliminate-empty-binding-form conservatively rejects reader prefixes");
    }
    if contains_headed_form(request.dialect, &form, "declare") {
        bail!("eliminate-empty-binding-form conservatively rejects declarations");
    }

    let body = &form.children[2..];
    let replacement = match body {
        [] => "nil".to_owned(),
        [only] => only.span.slice(request.input).to_owned(),
        many => format!(
            "(progn {})",
            many.iter()
                .map(|expression| expression.span.slice(request.input))
                .collect::<Vec<_>>()
                .join(" ")
        ),
    };
    let rewritten = replace_span(request.input, form.span, &replacement);
    SyntaxTree::parse(&rewritten).context("eliminate-empty-binding-form output is not valid")?;

    Ok(EliminateEmptyBindingFormPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: form.span,
        body_form_count: body.len(),
        introduced_progn: body.len() > 1,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn require_empty_binding_form(dialect: Dialect, form: &ExpressionView) -> Result<()> {
    if form.kind != ExpressionKind::List
        || !form.reader_prefixes.is_empty()
        || form.children.len() < 2
    {
        bail!("eliminate-empty-binding-form selected form must be a plain let or let* form");
    }
    let matches = form
        .children
        .first()
        .and_then(atom_symbol_text)
        .is_some_and(|head| {
            if dialect == Dialect::CommonLisp {
                common_lisp_symbol_reference_eq(head, "let")
                    || common_lisp_symbol_reference_eq(head, "let*")
            } else {
                matches!(head, "let" | "let*")
            }
        });
    if !matches
        || form.children[1].kind != ExpressionKind::List
        || !form.children[1].children.is_empty()
    {
        bail!("eliminate-empty-binding-form requires an empty binding list");
    }
    Ok(())
}

fn require_known_expression_context(
    tree: &SyntaxTree,
    path: &Path,
    dialect: Dialect,
) -> Result<()> {
    let indexes = path.to_raw_indexes();
    if indexes.len() < 2 {
        bail!("eliminate-empty-binding-form refuses top-level forms");
    }
    for depth in 1..indexes.len() {
        let ancestor = tree
            .select_path(&Path::from_indexes(indexes[..depth].to_vec()))?
            .view();
        if !ancestor.reader_prefixes.is_empty() {
            bail!("eliminate-empty-binding-form refuses reader-prefixed contexts");
        }
    }
    let child_index = *indexes.last().expect("non-empty path");
    let parent = tree
        .select_path(&Path::from_indexes(indexes[..indexes.len() - 1].to_vec()))?
        .view();
    let head = parent
        .children
        .first()
        .and_then(atom_symbol_text)
        .context("eliminate-empty-binding-form requires a known expression context")?;
    let is = |expected| {
        if dialect == Dialect::CommonLisp {
            common_lisp_symbol_reference_eq(head, expected)
        } else {
            head == expected
        }
    };
    let known = (is("progn") && child_index >= 1)
        || (is("if") && (1..=3).contains(&child_index))
        || ((is("when") || is("unless")) && child_index >= 1)
        || ((is("let") || is("let*")) && child_index >= 2)
        || (is("lambda") && child_index >= 2)
        || (is("defun") && child_index >= 3);
    if !known {
        bail!("eliminate-empty-binding-form requires a known expression position");
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

    fn request<'a>(
        input: &'a str,
        dialect: Dialect,
        path: &str,
    ) -> EliminateEmptyBindingFormRequest<'a> {
        EliminateEmptyBindingFormRequest {
            input,
            dialect,
            path: path.parse().expect("path"),
        }
    }

    #[test]
    fn eliminates_single_and_multiple_bodies() {
        let single = plan_eliminate_empty_binding_form(request(
            "(if ok (let () value) nil)",
            Dialect::CommonLisp,
            "0.2",
        ))
        .expect("plan");
        assert_eq!(single.rewritten, "(if ok value nil)");
        let multiple = plan_eliminate_empty_binding_form(request(
            "(progn (let* () (first) (second)))",
            Dialect::EmacsLisp,
            "0.1",
        ))
        .expect("plan");
        assert_eq!(multiple.rewritten, "(progn (progn (first) (second)))");
        assert!(multiple.introduced_progn);
    }

    #[test]
    fn rejects_top_level_unknown_context_and_declarations() {
        assert!(plan_eliminate_empty_binding_form(request(
            "(let () value)",
            Dialect::CommonLisp,
            "0"
        ))
        .is_err());
        assert!(plan_eliminate_empty_binding_form(request(
            "(unknown (let () value))",
            Dialect::EmacsLisp,
            "0.1"
        ))
        .is_err());
        assert!(plan_eliminate_empty_binding_form(request(
            "(progn (let () (declare (special x)) x))",
            Dialect::CommonLisp,
            "0.1"
        ))
        .is_err());
    }
}
