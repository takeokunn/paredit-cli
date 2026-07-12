//! Convert a parallel `let` into `let*` when doing so preserves name resolution.

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{
    ByteSpan, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};
use anyhow::{Context, Result, bail};

#[derive(Debug, Clone)]
pub struct ConvertLetToLetStarRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct ConvertLetToLetStarPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub binding_names: Vec<SymbolName>,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_convert_let_to_let_star(
    request: ConvertLetToLetStarRequest<'_>,
) -> Result<ConvertLetToLetStarPlan> {
    if !matches!(request.dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("convert-let-to-let-star supports only Common Lisp and Emacs Lisp");
    }
    let tree =
        SyntaxTree::parse(request.input).context("convert-let-to-let-star input is not valid")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    if tree.has_comment_in(form.span) {
        bail!("convert-let-to-let-star cannot rewrite a form containing comments");
    }
    require_head(request.dialect, &form, "let")?;
    if contains_headed_form(request.dialect, &form, "declare") {
        bail!("convert-let-to-let-star conservatively rejects declarations");
    }
    let bindings = form
        .children
        .get(1)
        .context("convert-let-to-let-star requires a binding list")?;
    if bindings.kind != ExpressionKind::List || !bindings.reader_prefixes.is_empty() {
        bail!("convert-let-to-let-star requires a plain binding list");
    }
    let mut names = Vec::with_capacity(bindings.children.len());
    let mut initializers = Vec::with_capacity(bindings.children.len());
    for binding in &bindings.children {
        let (name, initializer) = parse_binding(binding)?;
        if names
            .iter()
            .any(|old: &SymbolName| symbol_eq(request.dialect, old.as_str(), name.as_str()))
        {
            bail!("convert-let-to-let-star requires unique binding names");
        }
        names.push(name);
        initializers.push(initializer);
    }
    for (index, initializer) in initializers.iter().enumerate() {
        let Some(initializer) = initializer else {
            continue;
        };
        for earlier in &names[..index] {
            let mut references = Vec::new();
            collect_unshadowed_symbol_references(
                request.dialect,
                initializer,
                earlier,
                request.input,
                &mut references,
            );
            if !references.is_empty() {
                bail!(
                    "initializer for '{}' references earlier binding '{}'",
                    names[index],
                    earlier
                );
            }
        }
    }
    let rewritten = replace_span(request.input, form.children[0].span, "let*");
    SyntaxTree::parse(&rewritten).context("convert-let-to-let-star output is not valid")?;
    Ok(ConvertLetToLetStarPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: form.span,
        binding_names: names,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn parse_binding(binding: &ExpressionView) -> Result<(SymbolName, Option<ExpressionView>)> {
    if binding.kind == ExpressionKind::Atom {
        return Ok((plain_symbol(binding)?, None));
    }
    if binding.kind != ExpressionKind::List
        || !binding.reader_prefixes.is_empty()
        || !(1..=2).contains(&binding.children.len())
    {
        bail!("convert-let-to-let-star requires plain, non-destructuring bindings");
    }
    Ok((
        plain_symbol(&binding.children[0])?,
        binding.children.get(1).cloned(),
    ))
}

fn plain_symbol(view: &ExpressionView) -> Result<SymbolName> {
    if view.kind != ExpressionKind::Atom || !view.reader_prefixes.is_empty() {
        bail!("convert-let-to-let-star requires a plain binding name");
    }
    SymbolName::new(
        atom_symbol_text(view).context("convert-let-to-let-star requires a plain binding name")?,
    )
    .context("invalid binding name")
}

fn symbol_eq(dialect: Dialect, left: &str, right: &str) -> bool {
    if dialect == Dialect::CommonLisp {
        common_lisp_symbol_reference_eq(left, right)
    } else {
        left == right
    }
}

fn require_head(dialect: Dialect, view: &ExpressionView, expected: &str) -> Result<()> {
    if view.kind != ExpressionKind::List
        || !view.reader_prefixes.is_empty()
        || !view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| symbol_eq(dialect, head, expected))
    {
        bail!("convert-let-to-let-star selected form must be a plain let form");
    }
    Ok(())
}

fn contains_headed_form(dialect: Dialect, view: &ExpressionView, expected: &str) -> bool {
    (view.kind == ExpressionKind::List
        && view.reader_prefixes.is_empty()
        && view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| symbol_eq(dialect, head, expected)))
        || view
            .children
            .iter()
            .any(|child| contains_headed_form(dialect, child, expected))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn req(input: &str, dialect: Dialect) -> ConvertLetToLetStarRequest<'_> {
        ConvertLetToLetStarRequest {
            input,
            dialect,
            path: "0".parse().unwrap(),
        }
    }
    #[test]
    fn converts_both_dialects() {
        for d in [Dialect::CommonLisp, Dialect::EmacsLisp] {
            assert_eq!(
                plan_convert_let_to_let_star(req("(let ((x 1) (y 2)) (+ x y))", d))
                    .unwrap()
                    .rewritten,
                "(let* ((x 1) (y 2)) (+ x y))"
            );
        }
    }
    #[test]
    fn dependency_is_rejected_but_shadowing_is_safe() {
        for d in [Dialect::CommonLisp, Dialect::EmacsLisp] {
            assert!(plan_convert_let_to_let_star(req("(let ((x 1) (y (+ x 2))) y)", d)).is_err());
            assert!(
                plan_convert_let_to_let_star(req("(let ((x 1) (y (let ((x 2)) x))) y)", d)).is_ok()
            );
        }
    }
    #[test]
    fn unsafe_syntax_is_rejected() {
        assert!(
            plan_convert_let_to_let_star(req("(let ((x 1) (X 2)) x)", Dialect::CommonLisp))
                .is_err()
        );
        assert!(
            plan_convert_let_to_let_star(req("(let ((x 1)) ; c\n x)", Dialect::EmacsLisp)).is_err()
        );
        assert!(
            plan_convert_let_to_let_star(req(
                "(let ((x 1)) (declare (special x)) x)",
                Dialect::CommonLisp
            ))
            .is_err()
        );
        assert!(plan_convert_let_to_let_star(req("'(let ((x 1)) x)", Dialect::EmacsLisp)).is_err());
    }
}
