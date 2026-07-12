//! Use case for converting a non-recursive Common Lisp `labels` form into `flet`.

use anyhow::{Context, Result, bail};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, Path, SyntaxTree};

#[derive(Debug, Clone)]
pub struct ConvertLabelsToFletRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct ConvertLabelsToFletPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub binding_count: usize,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_convert_labels_to_flet(
    request: ConvertLabelsToFletRequest<'_>,
) -> Result<ConvertLabelsToFletPlan> {
    if request.dialect != Dialect::CommonLisp {
        bail!("convert-labels-to-flet supports only Common Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("convert-labels-to-flet input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    if tree.has_comment_in(form.span) {
        bail!("convert-labels-to-flet cannot rewrite a form containing comments");
    }
    if contains_reader_prefix(&form) {
        bail!("convert-labels-to-flet requires a form without reader prefixes");
    }
    let head = require_labels_form(&form)?;
    let bindings = form
        .children
        .get(1)
        .context("convert-labels-to-flet requires a binding list")?;
    if bindings.kind != ExpressionKind::List {
        bail!("convert-labels-to-flet requires a plain binding list");
    }

    let mut names: Vec<String> = Vec::with_capacity(bindings.children.len());
    for definition in &bindings.children {
        if definition.kind != ExpressionKind::List || definition.children.len() < 2 {
            bail!("convert-labels-to-flet requires plain local function definitions");
        }
        let name = plain_atom(&definition.children[0])
            .context("convert-labels-to-flet requires a plain local function name")?;
        if definition.children[1].kind != ExpressionKind::List {
            bail!("convert-labels-to-flet requires a plain lambda list");
        }
        if names
            .iter()
            .any(|existing| common_lisp_symbol_reference_eq(existing, name))
        {
            bail!("convert-labels-to-flet requires unique local function names");
        }
        names.push(name.to_owned());
    }

    for definition in &bindings.children {
        for body in definition.children.iter().skip(2) {
            if contains_local_function_reference(body, &names) {
                bail!(
                    "convert-labels-to-flet cannot convert recursive or mutually recursive definitions"
                );
            }
        }
    }

    let replacement_head = replace_labels_name(head);
    let rewritten = replace_span(request.input, form.children[0].span, &replacement_head);
    SyntaxTree::parse(&rewritten)
        .context("convert-labels-to-flet output is not a valid S-expression document")?;

    Ok(ConvertLabelsToFletPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: form.span,
        binding_count: names.len(),
        changed: rewritten != request.input,
        rewritten,
    })
}

fn require_labels_form(form: &ExpressionView) -> Result<&str> {
    if form.kind != ExpressionKind::List || form.children.len() < 2 {
        bail!("convert-labels-to-flet selected form must be a labels form");
    }
    let head = plain_atom(&form.children[0])
        .context("convert-labels-to-flet selected form must have a plain head")?;
    if !common_lisp_symbol_reference_eq(head, "labels") {
        bail!("convert-labels-to-flet selected form must be a labels form");
    }
    Ok(head)
}

fn plain_atom(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom && view.reader_prefixes.is_empty())
        .then(|| atom_symbol_text(view))
        .flatten()
}

fn contains_reader_prefix(view: &ExpressionView) -> bool {
    !view.reader_prefixes.is_empty() || view.children.iter().any(contains_reader_prefix)
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

fn replace_labels_name(head: &str) -> String {
    match head.rsplit_once(':') {
        Some((package, _)) => format!("{package}:flet"),
        None => "flet".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(input: &str, dialect: Dialect) -> ConvertLabelsToFletRequest<'_> {
        ConvertLabelsToFletRequest {
            input,
            dialect,
            path: "0".parse().expect("path"),
        }
    }

    #[test]
    fn converts_non_recursive_labels_and_allows_body_calls() {
        let input = "(labels ((parse (x) (list x)) (emit (x) (print x))) (parse value))";
        let plan = plan_convert_labels_to_flet(request(input, Dialect::CommonLisp)).expect("plan");
        assert_eq!(
            plan.rewritten,
            "(flet ((parse (x) (list x)) (emit (x) (print x))) (parse value))"
        );
        assert_eq!(plan.binding_count, 2);
    }

    #[test]
    fn preserves_a_package_qualified_head() {
        let plan = plan_convert_labels_to_flet(request(
            "(cl:labels ((work () 1)) (work))",
            Dialect::CommonLisp,
        ))
        .expect("plan");
        assert_eq!(plan.rewritten, "(cl:flet ((work () 1)) (work))");
    }

    #[test]
    fn rejects_self_and_mutual_recursion() {
        assert!(
            plan_convert_labels_to_flet(request(
                "(labels ((walk (x) (walk x))) (walk value))",
                Dialect::CommonLisp,
            ))
            .is_err()
        );
        assert!(
            plan_convert_labels_to_flet(request(
                "(labels ((walk () (function walk))) (walk))",
                Dialect::CommonLisp,
            ))
            .is_err()
        );
        assert!(
            plan_convert_labels_to_flet(request(
                "(labels ((left () (right)) (right () 1)) (left))",
                Dialect::CommonLisp,
            ))
            .is_err()
        );
    }

    #[test]
    fn rejects_comments_reader_forms_and_malformed_definitions() {
        for input in [
            "(labels ((work () ; keep\n 1)) (work))",
            "(labels ((work () #'identity)) (work))",
            "(labels (work) (work))",
            "(labels ((work value 1)) (work))",
            "'(labels ((work () 1)) (work))",
            "(labels #+sbcl ((work () 1)) (work))",
        ] {
            assert!(
                plan_convert_labels_to_flet(request(input, Dialect::CommonLisp)).is_err(),
                "unexpectedly accepted {input}"
            );
        }
    }

    #[test]
    fn rejects_non_labels_and_other_dialects() {
        assert!(
            plan_convert_labels_to_flet(request(
                "(flet ((work () 1)) (work))",
                Dialect::CommonLisp,
            ))
            .is_err()
        );
        assert!(
            plan_convert_labels_to_flet(request(
                "(labels ((work () 1)) (work))",
                Dialect::EmacsLisp,
            ))
            .is_err()
        );
    }
}
