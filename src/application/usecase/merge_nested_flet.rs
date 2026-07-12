//! Merge directly nested Common Lisp `flet` forms without changing function scope.

use anyhow::{Context, Result, bail};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct MergeNestedFletRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct MergeNestedFletPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub outer_binding_count: usize,
    pub inner_binding_count: usize,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_merge_nested_flet(request: MergeNestedFletRequest<'_>) -> Result<MergeNestedFletPlan> {
    if request.dialect != Dialect::CommonLisp {
        bail!("merge-nested-flet supports only Common Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("merge-nested-flet input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let outer = tree.select_path(&request.path)?.view();
    require_flet(&outer, "selected form")?;
    reject_unsafe_syntax(&tree, &outer)?;
    if outer.children.len() != 3 {
        bail!("merge-nested-flet requires the outer body to contain only one form");
    }

    let outer_bindings = require_definitions(&outer.children[1], "outer")?;
    let inner = &outer.children[2];
    require_flet(inner, "outer body")?;
    if inner.children.len() < 3 {
        bail!("merge-nested-flet requires the inner flet to have a body");
    }
    let inner_bindings = require_definitions(&inner.children[1], "inner")?;
    let outer_names = definition_names(outer_bindings)?;
    let inner_names = definition_names(inner_bindings)?;
    ensure_unique_names(&outer_names, &inner_names)?;

    for definition in &inner_bindings.children {
        for body in definition.children.iter().skip(2) {
            if contains_local_function_reference(body, &outer_names) {
                bail!(
                    "merge-nested-flet cannot move an inner definition outside the scope of an outer local function"
                );
            }
        }
    }

    let outer_text = list_contents(request.input, outer_bindings);
    let inner_text = list_contents(request.input, inner_bindings);
    let separator = if outer_text.trim().is_empty() || inner_text.trim().is_empty() {
        ""
    } else {
        " "
    };
    let head = outer.children[0].span.slice(request.input);
    let body = &request.input[inner_bindings.span.end().get()..inner.span.end().get() - 1];
    let replacement = format!("({head} ({outer_text}{separator}{inner_text}){body})");
    let rewritten = replace_span(request.input, outer.span, &replacement);
    SyntaxTree::parse(&rewritten).context("merge-nested-flet output is not valid")?;

    Ok(MergeNestedFletPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: outer.span,
        outer_binding_count: outer_bindings.children.len(),
        inner_binding_count: inner_bindings.children.len(),
        changed: rewritten != request.input,
        rewritten,
    })
}

fn require_flet(view: &ExpressionView, role: &str) -> Result<()> {
    if view.kind != ExpressionKind::List
        || !view.reader_prefixes.is_empty()
        || !view
            .children
            .first()
            .and_then(plain_atom)
            .is_some_and(|head| common_lisp_symbol_reference_eq(head, "flet"))
    {
        bail!("merge-nested-flet {role} must be a plain flet form");
    }
    Ok(())
}

fn require_definitions<'a>(view: &'a ExpressionView, role: &str) -> Result<&'a ExpressionView> {
    if view.kind != ExpressionKind::List || !view.reader_prefixes.is_empty() {
        bail!("merge-nested-flet requires a plain {role} definition list");
    }
    for definition in &view.children {
        if definition.kind != ExpressionKind::List
            || !definition.reader_prefixes.is_empty()
            || definition.children.len() < 2
            || plain_atom(&definition.children[0]).is_none()
            || definition.children[1].kind != ExpressionKind::List
            || !definition.children[1].reader_prefixes.is_empty()
        {
            bail!("merge-nested-flet requires plain local function definitions");
        }
    }
    Ok(view)
}

fn definition_names(definitions: &ExpressionView) -> Result<Vec<String>> {
    definitions
        .children
        .iter()
        .map(|definition| {
            plain_atom(&definition.children[0])
                .map(str::to_owned)
                .context("merge-nested-flet requires a plain local function name")
        })
        .collect()
}

fn ensure_unique_names(outer: &[String], inner: &[String]) -> Result<()> {
    let mut names: Vec<&str> = Vec::with_capacity(outer.len() + inner.len());
    for name in outer.iter().chain(inner) {
        if names
            .iter()
            .any(|existing| common_lisp_symbol_reference_eq(existing, name))
        {
            bail!("merge-nested-flet requires unique local function names");
        }
        names.push(name);
    }
    Ok(())
}

fn reject_unsafe_syntax(tree: &SyntaxTree, form: &ExpressionView) -> Result<()> {
    if tree.has_comment_in(form.span) {
        bail!("merge-nested-flet cannot rewrite a form containing comments");
    }
    if contains_reader_prefix(form) {
        bail!("merge-nested-flet conservatively rejects reader prefixes");
    }
    if contains_headed_form(form, "declare") {
        bail!("merge-nested-flet conservatively rejects declarations");
    }
    Ok(())
}

fn plain_atom(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom && view.reader_prefixes.is_empty())
        .then(|| atom_symbol_text(view))
        .flatten()
}

fn contains_reader_prefix(view: &ExpressionView) -> bool {
    !view.reader_prefixes.is_empty() || view.children.iter().any(contains_reader_prefix)
}

fn contains_headed_form(view: &ExpressionView, expected: &str) -> bool {
    (view.kind == ExpressionKind::List
        && view
            .children
            .first()
            .and_then(plain_atom)
            .is_some_and(|head| common_lisp_symbol_reference_eq(head, expected)))
        || view
            .children
            .iter()
            .any(|child| contains_headed_form(child, expected))
}

fn contains_local_function_reference(view: &ExpressionView, names: &[String]) -> bool {
    if view.kind == ExpressionKind::List {
        let head = view.children.first().and_then(plain_atom);
        if head.is_some_and(|head| {
            names
                .iter()
                .any(|name| common_lisp_symbol_reference_eq(name, head))
        }) {
            return true;
        }
        if head.is_some_and(|head| common_lisp_symbol_reference_eq(head, "function"))
            && view
                .children
                .get(1)
                .and_then(plain_atom)
                .is_some_and(|name| {
                    names
                        .iter()
                        .any(|local| common_lisp_symbol_reference_eq(local, name))
                })
        {
            return true;
        }
    }
    view.children
        .iter()
        .any(|child| contains_local_function_reference(child, names))
}

fn list_contents<'a>(input: &'a str, view: &ExpressionView) -> &'a str {
    &input[view.span.start().get() + 1..view.span.end().get() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(input: &str, dialect: Dialect) -> MergeNestedFletRequest<'_> {
        MergeNestedFletRequest {
            input,
            dialect,
            path: "0".parse().expect("path"),
        }
    }

    #[test]
    fn merges_independent_nested_flet_definitions() {
        let plan = plan_merge_nested_flet(request(
            "(flet ((parse (x) (list x))) (flet ((emit (x) (print x))) (emit (parse value))))",
            Dialect::CommonLisp,
        ))
        .expect("plan");
        assert_eq!(
            plan.rewritten,
            "(flet ((parse (x) (list x)) (emit (x) (print x))) (emit (parse value)))"
        );
        assert_eq!(plan.outer_binding_count, 1);
        assert_eq!(plan.inner_binding_count, 1);
    }

    #[test]
    fn rejects_inner_definition_references_to_outer_functions() {
        for input in [
            "(flet ((parse (x) x)) (flet ((emit (x) (parse x))) (emit value)))",
            "(flet ((parse (x) x)) (flet ((emit () (function parse))) (emit)))",
        ] {
            assert!(
                plan_merge_nested_flet(request(input, Dialect::CommonLisp)).is_err(),
                "unexpectedly accepted {input}"
            );
        }
    }

    #[test]
    fn rejects_duplicates_non_single_bodies_and_ambiguous_syntax() {
        for input in [
            "(flet ((work () 1)) (flet ((work () 2)) (work)))",
            "(flet ((left () 1)) (left) (flet ((right () 2)) (right)))",
            "(flet (((setf item) (value) value)) (flet ((read () 1)) (read)))",
            "(flet ((left () 1)) (flet ((right () (declare (inline left)) 2)) (right)))",
            "(flet ((left () ; keep\n 1)) (flet ((right () 2)) (right)))",
            "(flet ((left () #'identity)) (flet ((right () 2)) (right)))",
        ] {
            assert!(
                plan_merge_nested_flet(request(input, Dialect::CommonLisp)).is_err(),
                "unexpectedly accepted {input}"
            );
        }
        assert!(
            plan_merge_nested_flet(request(
                "(flet ((left () 1)) (flet ((right () 2)) (right)))",
                Dialect::EmacsLisp,
            ))
            .is_err()
        );
    }
}
