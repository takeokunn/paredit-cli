//! Safe conversion of sequential Common Lisp iteration/binding forms.

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
pub struct ConvertSequentialBindingRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct ConvertSequentialBindingPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub binding_names: Vec<SymbolName>,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_convert_do_star_to_do(
    request: ConvertSequentialBindingRequest<'_>,
) -> Result<ConvertSequentialBindingPlan> {
    plan_conversion(request, Conversion::Do)
}

pub fn plan_convert_prog_star_to_prog(
    request: ConvertSequentialBindingRequest<'_>,
) -> Result<ConvertSequentialBindingPlan> {
    plan_conversion(request, Conversion::Prog)
}

#[derive(Clone, Copy)]
enum Conversion {
    Do,
    Prog,
}

impl Conversion {
    fn command(self) -> &'static str {
        match self {
            Self::Do => "convert-do-star-to-do",
            Self::Prog => "convert-prog-star-to-prog",
        }
    }

    fn source_head(self) -> &'static str {
        match self {
            Self::Do => "do*",
            Self::Prog => "prog*",
        }
    }

    fn target_head(self) -> &'static str {
        match self {
            Self::Do => "do",
            Self::Prog => "prog",
        }
    }

    fn max_binding_parts(self) -> usize {
        match self {
            Self::Do => 3,
            Self::Prog => 2,
        }
    }
}

fn plan_conversion(
    request: ConvertSequentialBindingRequest<'_>,
    conversion: Conversion,
) -> Result<ConvertSequentialBindingPlan> {
    let command = conversion.command();
    if request.dialect != Dialect::CommonLisp {
        bail!("{command} currently supports only Common Lisp");
    }
    let tree = SyntaxTree::parse(request.input)
        .with_context(|| format!("{command} input is not a valid S-expression document"))?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let form = tree.select_path(&request.path)?.view();
    if tree.has_comment_in(form.span) {
        bail!("{command} cannot rewrite a form containing comments");
    }
    if contains_reader_prefix(&form) {
        bail!("{command} conservatively rejects reader-prefixed syntax");
    }
    require_head(&form, conversion)?;
    if contains_headed_form(&form, "declare") {
        bail!("{command} conservatively rejects declarations");
    }
    let minimum_children = match conversion {
        Conversion::Do => 3,
        Conversion::Prog => 2,
    };
    if form.children.len() < minimum_children {
        bail!("{command} selected form is malformed");
    }

    let bindings = &form.children[1];
    if bindings.kind != ExpressionKind::List || !bindings.reader_prefixes.is_empty() {
        bail!("{command} requires a plain binding list");
    }
    if matches!(conversion, Conversion::Do) && form.children[2].kind != ExpressionKind::List {
        bail!("{command} requires a termination clause");
    }

    let mut names = Vec::with_capacity(bindings.children.len());
    let mut initializers = Vec::with_capacity(bindings.children.len());
    let mut steps = Vec::with_capacity(bindings.children.len());
    for binding in &bindings.children {
        let (name, initializer, step) = parse_binding(binding, conversion)?;
        if names.iter().any(|existing: &SymbolName| {
            common_lisp_symbol_reference_eq(existing.as_str(), name.as_str())
        }) {
            bail!("{command} requires unique binding names");
        }
        names.push(name);
        initializers.push(initializer);
        steps.push(step);
    }

    reject_dependencies(
        request.input,
        request.dialect,
        &names,
        &initializers,
        "initializer",
    )?;
    if matches!(conversion, Conversion::Do) {
        reject_dependencies(
            request.input,
            request.dialect,
            &names,
            &steps,
            "step expression",
        )?;
    }

    let head = &form.children[0];
    let rewritten = replace_span(request.input, head.span, conversion.target_head());
    SyntaxTree::parse(&rewritten)
        .with_context(|| format!("{command} output is not a valid S-expression document"))?;
    Ok(ConvertSequentialBindingPlan {
        dialect: request.dialect,
        path: request.path,
        form_span: form.span,
        binding_names: names,
        changed: rewritten != request.input,
        rewritten,
    })
}

fn parse_binding(
    binding: &ExpressionView,
    conversion: Conversion,
) -> Result<(SymbolName, Option<ExpressionView>, Option<ExpressionView>)> {
    let command = conversion.command();
    if binding.kind == ExpressionKind::Atom {
        return Ok((plain_symbol(binding, command)?, None, None));
    }
    if binding.kind != ExpressionKind::List || !binding.reader_prefixes.is_empty() {
        bail!("{command} requires plain variable bindings");
    }
    if !(1..=conversion.max_binding_parts()).contains(&binding.children.len()) {
        bail!("{command} rejects destructuring or malformed bindings");
    }
    let name = plain_symbol(&binding.children[0], command)?;
    Ok((
        name,
        binding.children.get(1).cloned(),
        binding.children.get(2).cloned(),
    ))
}

fn reject_dependencies(
    input: &str,
    dialect: Dialect,
    names: &[SymbolName],
    expressions: &[Option<ExpressionView>],
    role: &str,
) -> Result<()> {
    for (index, expression) in expressions.iter().enumerate() {
        let Some(expression) = expression else {
            continue;
        };
        for earlier in &names[..index] {
            let mut references = Vec::new();
            collect_unshadowed_symbol_references(
                dialect,
                expression,
                earlier,
                input,
                &mut references,
            );
            if !references.is_empty() {
                bail!(
                    "{role} for '{}' references earlier binding '{}'",
                    names[index],
                    earlier
                );
            }
        }
    }
    Ok(())
}

fn plain_symbol(view: &ExpressionView, command: &str) -> Result<SymbolName> {
    if view.kind != ExpressionKind::Atom || !view.reader_prefixes.is_empty() {
        bail!("{command} requires a plain binding name");
    }
    let text = atom_symbol_text(view)
        .with_context(|| format!("{command} requires a plain binding name"))?;
    SymbolName::new(text).context("invalid binding name")
}

fn require_head(view: &ExpressionView, conversion: Conversion) -> Result<()> {
    let command = conversion.command();
    let expected = conversion.source_head();
    if view.kind != ExpressionKind::List || !view.reader_prefixes.is_empty() {
        bail!("{command} selected form must be a plain {expected} form");
    }
    let matches = view
        .children
        .first()
        .and_then(atom_symbol_text)
        .is_some_and(|head| common_lisp_symbol_reference_eq(head, expected));
    if !matches {
        bail!("{command} selected form must be a {expected} form");
    }
    Ok(())
}

fn contains_reader_prefix(view: &ExpressionView) -> bool {
    !view.reader_prefixes.is_empty() || view.children.iter().any(contains_reader_prefix)
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

    fn request(input: &str) -> ConvertSequentialBindingRequest<'_> {
        ConvertSequentialBindingRequest {
            input,
            dialect: Dialect::CommonLisp,
            path: "0".parse().expect("path"),
        }
    }

    #[test]
    fn converts_independent_do_star() {
        let input = "(do* ((x (first) (next x)) (y (second) (next y))) ((done-p x y) y) (work x))";
        let plan = plan_convert_do_star_to_do(request(input)).expect("plan");
        assert_eq!(plan.rewritten, input.replacen("do*", "do", 1));
    }

    #[test]
    fn rejects_do_initializer_and_step_dependencies() {
        assert!(
            plan_convert_do_star_to_do(request("(do* ((x 1) (y (+ x 1))) ((done-p)))")).is_err()
        );
        let error =
            plan_convert_do_star_to_do(request("(do* ((x 1 (next x)) (y 2 (+ x y))) ((done-p)))"))
                .expect_err("step dependency");
        assert!(error.to_string().contains("step expression for 'y'"));
    }

    #[test]
    fn converts_independent_prog_star() {
        let input = "(prog* ((x (first)) (y (second))) start (return (+ x y)))";
        let plan = plan_convert_prog_star_to_prog(request(input)).expect("plan");
        assert_eq!(plan.rewritten, input.replacen("prog*", "prog", 1));
    }

    #[test]
    fn rejects_prog_dependency_and_ambiguous_syntax() {
        assert!(
            plan_convert_prog_star_to_prog(request("(prog* ((x 1) (y (+ x 1))) (return y))"))
                .is_err()
        );
        assert!(
            plan_convert_prog_star_to_prog(request(
                "(prog* ((x 1)) (declare (special x)) (return x))"
            ))
            .is_err()
        );
        assert!(plan_convert_do_star_to_do(request("(do* ((x '#.(value))) ((done-p)))")).is_err());
    }
}
