//! Merge directly nested parallel `let` forms when initializer scope is unchanged.

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
pub struct MergeNestedLetRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct MergeNestedLetPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub outer_binding_count: usize,
    pub inner_binding_count: usize,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_merge_nested_let(request: MergeNestedLetRequest<'_>) -> Result<MergeNestedLetPlan> {
    require_dialect(request.dialect)?;
    let tree = SyntaxTree::parse(request.input)
        .context("merge-nested-let input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let outer = tree.select_path(&request.path)?.view();
    require_let(request.dialect, &outer, "selected form")?;
    reject_unsafe_syntax(&tree, request.dialect, &outer)?;
    if outer.children.len() != 3 {
        bail!("merge-nested-let requires the outer body to contain only one form");
    }

    let outer_bindings = require_binding_list(&outer.children[1], "outer")?;
    let inner = &outer.children[2];
    require_let(request.dialect, inner, "outer body")?;
    if inner.children.len() < 3 {
        bail!("merge-nested-let requires the inner let to have a body");
    }
    let inner_bindings = require_binding_list(&inner.children[1], "inner")?;
    let outer_parsed = parse_bindings_with_initializers(outer_bindings)?;
    let outer_names = outer_parsed
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    ensure_unique_names(request.dialect, &outer_names)?;
    let inner_bindings_parsed = parse_bindings_with_initializers(inner_bindings)?;

    let mut all_names = outer_names.clone();
    for (name, initializer) in &inner_bindings_parsed {
        if all_names
            .iter()
            .any(|old| symbol_eq(request.dialect, old.as_str(), name.as_str()))
        {
            bail!("merge-nested-let requires unique binding names");
        }
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
                bail!("inner initializer for '{name}' references outer binding '{outer_name}'");
            }
        }
        all_names.push(name.clone());
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
    SyntaxTree::parse(&rewritten).context("merge-nested-let output is not valid")?;
    Ok(MergeNestedLetPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: outer.span,
        outer_binding_count: outer_bindings.children.len(),
        inner_binding_count: inner_bindings.children.len(),
        changed: rewritten != request.input,
        rewritten,
    })
}

fn require_dialect(dialect: Dialect) -> Result<()> {
    if !matches!(dialect, Dialect::CommonLisp | Dialect::EmacsLisp) {
        bail!("merge-nested-let supports only Common Lisp and Emacs Lisp");
    }
    Ok(())
}

fn require_let(dialect: Dialect, view: &ExpressionView, role: &str) -> Result<()> {
    if view.kind != ExpressionKind::List
        || !view.reader_prefixes.is_empty()
        || !view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| symbol_eq(dialect, head, "let"))
    {
        bail!("merge-nested-let {role} must be a plain let form");
    }
    Ok(())
}

fn require_binding_list<'a>(view: &'a ExpressionView, role: &str) -> Result<&'a ExpressionView> {
    if view.kind != ExpressionKind::List || !view.reader_prefixes.is_empty() {
        bail!("merge-nested-let requires a plain {role} binding list");
    }
    Ok(view)
}

fn parse_bindings_with_initializers(
    bindings: &ExpressionView,
) -> Result<Vec<(SymbolName, Option<ExpressionView>)>> {
    let mut result = Vec::new();
    for binding in &bindings.children {
        let (name, initializer) = if binding.kind == ExpressionKind::Atom {
            (plain_symbol(binding)?, None)
        } else if binding.kind == ExpressionKind::List
            && binding.reader_prefixes.is_empty()
            && (1..=2).contains(&binding.children.len())
        {
            (
                plain_symbol(&binding.children[0])?,
                binding.children.get(1).cloned(),
            )
        } else {
            bail!("merge-nested-let requires plain, non-destructuring bindings");
        };
        result.push((name, initializer));
    }
    Ok(result)
}

fn ensure_unique_names(dialect: Dialect, names: &[SymbolName]) -> Result<()> {
    for (index, name) in names.iter().enumerate() {
        if names[..index]
            .iter()
            .any(|old| symbol_eq(dialect, old.as_str(), name.as_str()))
        {
            bail!("merge-nested-let requires unique binding names");
        }
    }
    Ok(())
}

fn plain_symbol(view: &ExpressionView) -> Result<SymbolName> {
    if view.kind != ExpressionKind::Atom || !view.reader_prefixes.is_empty() {
        bail!("merge-nested-let requires a plain binding name");
    }
    let text = atom_symbol_text(view).context("merge-nested-let requires a plain binding name")?;
    SymbolName::new(text).context("invalid binding name")
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
        bail!("merge-nested-let cannot rewrite a form containing comments");
    }
    if contains_reader_prefix(form) {
        bail!("merge-nested-let conservatively rejects reader prefixes");
    }
    if contains_headed_form(dialect, form, "declare") {
        bail!("merge-nested-let conservatively rejects declarations");
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

fn list_contents<'a>(input: &'a str, view: &ExpressionView) -> &'a str {
    &input[view.span.start().get() + 1..view.span.end().get() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    fn req(input: &str, dialect: Dialect) -> MergeNestedLetRequest<'_> {
        MergeNestedLetRequest {
            input,
            dialect,
            path: "0".parse().unwrap(),
        }
    }
    #[test]
    fn merges_independent_nested_lets() {
        for dialect in [Dialect::CommonLisp, Dialect::EmacsLisp] {
            let plan =
                plan_merge_nested_let(req("(let ((x 1)) (let ((y 2)) (+ x y)))", dialect)).unwrap();
            assert_eq!(plan.rewritten, "(let ((x 1) (y 2)) (+ x y))");
        }
    }
    #[test]
    fn rejects_dependency_and_duplicate_name() {
        assert!(
            plan_merge_nested_let(req(
                "(let ((x 1)) (let ((y (+ x 1))) y))",
                Dialect::CommonLisp
            ))
            .is_err()
        );
        assert!(
            plan_merge_nested_let(req("(let ((x 1)) (let ((X 2)) x))", Dialect::CommonLisp))
                .is_err()
        );
    }
    #[test]
    fn accepts_shadowed_reference_in_initializer() {
        assert!(
            plan_merge_nested_let(req(
                "(let ((x 1)) (let ((y (let ((x 2)) x))) y))",
                Dialect::EmacsLisp
            ))
            .is_ok()
        );
    }

    #[test]
    fn rejects_comments_declarations_and_reader_prefixes() {
        assert!(
            plan_merge_nested_let(req(
                "(let ((x 1)) ; keep\n (let ((y 2)) y))",
                Dialect::EmacsLisp
            ))
            .is_err()
        );
        assert!(
            plan_merge_nested_let(req(
                "(let ((x 1)) (let ((y 2)) (declare (special y)) y))",
                Dialect::CommonLisp
            ))
            .is_err()
        );
        assert!(
            plan_merge_nested_let(req("(let ((x 'one)) (let ((y 2)) y))", Dialect::CommonLisp))
                .is_err()
        );
    }
}
