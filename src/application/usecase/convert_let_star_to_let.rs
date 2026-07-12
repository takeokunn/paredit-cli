//! Use case for converting an independent Common Lisp `let*` into `let`.

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
pub struct ConvertLetStarToLetRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct ConvertLetStarToLetPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub binding_names: Vec<SymbolName>,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_convert_let_star_to_let(
    request: ConvertLetStarToLetRequest<'_>,
) -> Result<ConvertLetStarToLetPlan> {
    if request.dialect != Dialect::CommonLisp {
        bail!("convert-let-star-to-let currently supports only Common Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .context("convert-let-star-to-let input is not a valid S-expression document")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    if tree.has_comment_in(form.span) {
        bail!("convert-let-star-to-let cannot rewrite a form containing comments");
    }
    require_head(&form, "let*")?;
    if contains_headed_form(&form, "declare") {
        bail!("convert-let-star-to-let conservatively rejects declarations");
    }

    let bindings = form
        .children
        .get(1)
        .context("convert-let-star-to-let requires a binding list")?;
    if bindings.kind != ExpressionKind::List || !bindings.reader_prefixes.is_empty() {
        bail!("convert-let-star-to-let requires a plain binding list");
    }

    let mut names = Vec::with_capacity(bindings.children.len());
    let mut initializers = Vec::with_capacity(bindings.children.len());
    for binding in &bindings.children {
        let (name, initializer) = parse_binding(binding)?;
        if names.iter().any(|existing: &SymbolName| {
            common_lisp_symbol_reference_eq(existing.as_str(), name.as_str())
        }) {
            bail!("convert-let-star-to-let requires unique binding names");
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

    let head = &form.children[0];
    let rewritten = replace_span(request.input, head.span, "let");
    SyntaxTree::parse(&rewritten)
        .context("convert-let-star-to-let output is not a valid S-expression document")?;
    Ok(ConvertLetStarToLetPlan {
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
        return Ok((plain_symbol(binding, "binding name")?, None));
    }
    if binding.kind != ExpressionKind::List || !binding.reader_prefixes.is_empty() {
        bail!("convert-let-star-to-let requires plain variable bindings");
    }
    if !(1..=2).contains(&binding.children.len()) {
        bail!("convert-let-star-to-let rejects destructuring or malformed bindings");
    }
    let name = plain_symbol(&binding.children[0], "binding name")?;
    Ok((name, binding.children.get(1).cloned()))
}

fn plain_symbol(view: &ExpressionView, role: &str) -> Result<SymbolName> {
    if view.kind != ExpressionKind::Atom || !view.reader_prefixes.is_empty() {
        bail!("convert-let-star-to-let requires a plain {role}");
    }
    let text = atom_symbol_text(view)
        .with_context(|| format!("convert-let-star-to-let requires a plain {role}"))?;
    SymbolName::new(text).with_context(|| format!("invalid {role}"))
}

fn require_head(view: &ExpressionView, expected: &str) -> Result<()> {
    if view.kind != ExpressionKind::List || !view.reader_prefixes.is_empty() {
        bail!("convert-let-star-to-let selected form must be a plain let* form");
    }
    let matches = view
        .children
        .first()
        .and_then(atom_symbol_text)
        .is_some_and(|head| common_lisp_symbol_reference_eq(head, expected));
    if !matches {
        bail!("convert-let-star-to-let selected form must be a let* form");
    }
    Ok(())
}

fn contains_headed_form(view: &ExpressionView, expected: &str) -> bool {
    let matches = view.kind == ExpressionKind::List
        && view.reader_prefixes.is_empty()
        && view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| common_lisp_symbol_reference_eq(head, expected));
    matches
        || view
            .children
            .iter()
            .any(|child| contains_headed_form(child, expected))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(input: &str) -> ConvertLetStarToLetRequest<'_> {
        ConvertLetStarToLetRequest {
            input,
            dialect: Dialect::CommonLisp,
            path: "0".parse().expect("path"),
        }
    }

    #[test]
    fn converts_independent_bindings_without_reformatting() {
        let input = "(let* ((x (first)) (y (second)))\n  (+ x y))";
        let plan = plan_convert_let_star_to_let(request(input)).expect("plan");
        assert_eq!(
            plan.rewritten,
            "(let ((x (first)) (y (second)))\n  (+ x y))"
        );
    }

    #[test]
    fn rejects_reference_to_earlier_binding() {
        let error = plan_convert_let_star_to_let(request("(let* ((x 1) (y (+ x 2))) y)"))
            .expect_err("dependency must fail");
        assert!(error.to_string().contains("references earlier binding 'x'"));
    }

    #[test]
    fn accepts_reference_shadowed_inside_initializer() {
        let input = "(let* ((x 1) (y (let ((x 2)) x))) y)";
        assert!(plan_convert_let_star_to_let(request(input)).is_ok());
    }

    #[test]
    fn rejects_destructuring_and_declarations() {
        assert!(plan_convert_let_star_to_let(request("(let* (((x y) pair)) x)")).is_err());
        assert!(
            plan_convert_let_star_to_let(request("(let* ((x 1)) (declare (special x)) x)"))
                .is_err()
        );
    }

    #[test]
    fn rejects_ambiguous_reader_constructs_and_duplicate_bindings() {
        assert!(plan_convert_let_star_to_let(request("(let* ((x 1)) ; keep\n x)")).is_err());
        assert!(plan_convert_let_star_to_let(request("(let* ((x #+sbcl 1)) x)")).is_err());
        assert!(plan_convert_let_star_to_let(request("(let* ((x 1) (X 2)) x)")).is_err());
    }
}
