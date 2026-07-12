//! Split a parallel `let` without capturing free initializer references.

use anyhow::{Context, Result, bail};

use crate::application::usecase::extract_shared::replace_span;
use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{
    ByteSpan, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

#[derive(Debug, Clone)]
pub struct SplitLetRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
    pub binding_index: usize,
}

#[derive(Debug, Clone)]
pub struct SplitLetPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub binding_index: usize,
    pub outer_binding_count: usize,
    pub inner_binding_count: usize,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_split_let(request: SplitLetRequest<'_>) -> Result<SplitLetPlan> {
    if !matches!(request.dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("split-let supports only Common Lisp and Emacs Lisp");
    }
    let tree = SyntaxTree::parse(request.input).context("split-let input is not valid")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    require_let(request.dialect, &form)?;
    reject_unsafe_syntax(&tree, request.dialect, &form)?;
    if form.children.len() < 3 {
        bail!("split-let requires a body");
    }
    let bindings = &form.children[1];
    if bindings.kind != ExpressionKind::List || !bindings.reader_prefixes.is_empty() {
        bail!("split-let requires a plain binding list");
    }
    if request.binding_index == 0 || request.binding_index >= bindings.children.len() {
        bail!(
            "split-let --binding-index must be between 1 and {}",
            bindings.children.len().saturating_sub(1)
        );
    }
    let parsed = parse_bindings(bindings)?;
    let outer_names = parsed[..request.binding_index]
        .iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    for (inner_name, initializer) in &parsed[request.binding_index..] {
        for outer_name in &outer_names {
            let mut references = Vec::new();
            if let Some(initializer) = initializer {
                collect_unshadowed_symbol_references(
                    request.dialect,
                    initializer,
                    outer_name,
                    request.input,
                    &mut references,
                );
            }
            if !references.is_empty() {
                bail!(
                    "splitting would capture reference to '{outer_name}' in initializer for '{inner_name}'"
                );
            }
        }
    }
    let head = form.children[0].span.slice(request.input);
    let outer = bindings.children[..request.binding_index]
        .iter()
        .map(|v| v.span.slice(request.input))
        .collect::<Vec<_>>()
        .join(" ");
    let inner = bindings.children[request.binding_index..]
        .iter()
        .map(|v| v.span.slice(request.input))
        .collect::<Vec<_>>()
        .join(" ");
    let body = &request.input[bindings.span.end().get()..form.span.end().get() - 1];
    let replacement = format!("({head} ({outer}) ({head} ({inner}){body}))");
    let rewritten = replace_span(request.input, form.span, &replacement);
    SyntaxTree::parse(&rewritten).context("split-let output is not valid")?;
    Ok(SplitLetPlan {
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

fn require_let(dialect: Dialect, form: &ExpressionView) -> Result<()> {
    if form.kind != ExpressionKind::List
        || !form.reader_prefixes.is_empty()
        || !form
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| symbol_eq(dialect, head, "let"))
    {
        bail!("split-let selected form must be a plain let form");
    }
    Ok(())
}

fn parse_bindings(bindings: &ExpressionView) -> Result<Vec<(SymbolName, Option<ExpressionView>)>> {
    bindings
        .children
        .iter()
        .map(|binding| {
            if binding.kind == ExpressionKind::Atom {
                Ok((plain_symbol(binding)?, None))
            } else if binding.kind == ExpressionKind::List
                && binding.reader_prefixes.is_empty()
                && (1..=2).contains(&binding.children.len())
            {
                Ok((
                    plain_symbol(&binding.children[0])?,
                    binding.children.get(1).cloned(),
                ))
            } else {
                bail!("split-let requires plain, non-destructuring bindings")
            }
        })
        .collect()
}

fn plain_symbol(view: &ExpressionView) -> Result<SymbolName> {
    if view.kind != ExpressionKind::Atom || !view.reader_prefixes.is_empty() {
        bail!("split-let requires a plain binding name");
    }
    SymbolName::new(atom_symbol_text(view).context("split-let requires a plain binding name")?)
        .context("invalid binding name")
}

fn symbol_eq(dialect: Dialect, left: &str, right: &str) -> bool {
    if dialect == Dialect::CommonLisp {
        common_lisp_symbol_reference_eq(left, right)
    } else {
        left == right
    }
}

fn reject_unsafe_syntax(tree: &SyntaxTree, dialect: Dialect, form: &ExpressionView) -> Result<()> {
    if tree.has_comment_in(form.span) {
        bail!("split-let cannot rewrite a form containing comments");
    }
    if contains_reader_prefix(form) {
        bail!("split-let conservatively rejects reader prefixes");
    }
    if contains_headed_form(dialect, form, "declare") {
        bail!("split-let conservatively rejects declarations");
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
            .is_some_and(|head| symbol_eq(dialect, head, expected)))
        || view
            .children
            .iter()
            .any(|child| contains_headed_form(dialect, child, expected))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn req(input: &str, dialect: Dialect, binding_index: usize) -> SplitLetRequest<'_> {
        SplitLetRequest {
            input,
            dialect,
            path: "0".parse().unwrap(),
            binding_index,
        }
    }
    #[test]
    fn splits_when_initializers_remain_independent() {
        for dialect in [Dialect::CommonLisp, Dialect::EmacsLisp] {
            assert_eq!(
                plan_split_let(req("(let ((x 1) (y 2)) (+ x y))", dialect, 1))
                    .unwrap()
                    .rewritten,
                "(let ((x 1)) (let ((y 2)) (+ x y)))"
            );
        }
    }
    #[test]
    fn rejects_capture_but_accepts_shadowed_reference() {
        assert!(
            plan_split_let(req("(let ((x 1) (y (+ x 1))) y)", Dialect::CommonLisp, 1)).is_err()
        );
        assert!(
            plan_split_let(req(
                "(let ((x 1) (y (let ((x 2)) x))) y)",
                Dialect::EmacsLisp,
                1
            ))
            .is_ok()
        );
    }

    #[test]
    fn rejects_invalid_boundary_and_unsafe_syntax() {
        assert!(plan_split_let(req("(let ((x 1) (y 2)) y)", Dialect::CommonLisp, 0)).is_err());
        assert!(
            plan_split_let(req(
                "(let ((x 1) (y 2)) (declare (special x)) y)",
                Dialect::CommonLisp,
                1
            ))
            .is_err()
        );
        assert!(plan_split_let(req("(let ((x 'one) (y 2)) y)", Dialect::EmacsLisp, 1)).is_err());
    }
}
