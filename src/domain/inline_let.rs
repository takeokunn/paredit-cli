//! Pure planning rules for inlining a single-binding `let` form.

use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::lexical_scope::{collect_unshadowed_symbol_references, value_capture};
use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

#[derive(Debug, Clone)]
pub struct InlineLetRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub target: ExpressionView,
    pub allow_duplicate_evaluation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineLetPlan {
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub let_span: ByteSpan,
    pub binding_name: SymbolName,
    pub binding_value: String,
    pub body_count: usize,
    pub reference_count: usize,
    pub replacement: String,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_inline_let(request: InlineLetRequest<'_>) -> Result<InlineLetPlan> {
    let input_tree = SyntaxTree::parse(request.input)
        .context("inline-let input is not a valid S-expression document")?;
    crate::domain::mutation_safety::reject_common_lisp_reader_conditionals(
        &input_tree,
        request.dialect,
    )?;
    let plan = plan(CoreRequest {
        input: request.input,
        dialect: request.dialect,
        path: request.path,
        target: request.target,
        allow_duplicate_evaluation: request.allow_duplicate_evaluation,
    })?;
    Ok(InlineLetPlan {
        dialect: plan.dialect,
        path: plan.path,
        let_span: plan.let_span,
        binding_name: plan.binding_name,
        binding_value: plan.binding_value,
        body_count: plan.body_count,
        reference_count: plan.reference_count,
        replacement: plan.replacement,
        rewritten: plan.rewritten,
        changed: plan.changed,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct CoreRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub target: ExpressionView,
    pub allow_duplicate_evaluation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CorePlan {
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub let_span: ByteSpan,
    pub binding_name: SymbolName,
    pub binding_value: String,
    pub body_count: usize,
    pub reference_count: usize,
    pub replacement: String,
    pub rewritten: String,
    pub changed: bool,
}

pub(crate) fn plan(request: CoreRequest<'_>) -> Result<CorePlan> {
    let parts = parts(request.dialect, request.input, &request.target)?;
    let reference_count = parts.reference_spans.len();
    if reference_count == 0 {
        anyhow::bail!("inline-let would drop an unused binding value");
    }
    if reference_count > 1 && !request.allow_duplicate_evaluation {
        anyhow::bail!(
            "inline-let would duplicate binding value evaluation; pass --allow-duplicate-evaluation to permit it"
        );
    }

    let replacement = replace_body_references(
        request.input,
        parts.body_span,
        &parts.reference_spans,
        &parts.binding_value,
    );
    let rewritten = replace_span(request.input, parts.let_span, &replacement);
    SyntaxTree::parse(&rewritten)
        .context("inline-let output is not a valid S-expression document")?;

    Ok(CorePlan {
        dialect: request.dialect,
        path: request.path,
        let_span: parts.let_span,
        binding_name: parts.binding_name,
        binding_value: parts.binding_value,
        body_count: parts.body_count,
        reference_count,
        changed: rewritten != request.input,
        replacement,
        rewritten,
    })
}

struct Parts {
    let_span: ByteSpan,
    binding_name: SymbolName,
    binding_value: String,
    body_count: usize,
    body_span: ByteSpan,
    reference_spans: Vec<ByteSpan>,
}

fn parts(dialect: Dialect, input: &str, target: &ExpressionView) -> Result<Parts> {
    if target.kind != ExpressionKind::List {
        anyhow::bail!("inline-let selection must be a let list");
    }
    if target.children.len() < 3 {
        anyhow::bail!("inline-let requires one binding and at least one body expression");
    }
    let head = atom_text(&target.children[0]).context("inline-let form must start with an atom")?;
    if !dialect.supports_inline_let_refactor_head(head) {
        anyhow::bail!("inline-let selection must start with let");
    }

    let (binding_name, binding_value_view) = match dialect {
        Dialect::Clojure | Dialect::Janet | Dialect::Fennel => {
            vector_let_binding(&target.children[1])?
        }
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Scheme | Dialect::Unknown => {
            list_pair_let_binding(&target.children[1])?
        }
    };
    let binding_name = SymbolName::new(binding_name)?;
    let mut reference_spans = Vec::new();
    for body in &target.children[2..] {
        collect_unshadowed_symbol_references(
            dialect,
            body,
            &binding_name,
            input,
            &mut reference_spans,
        );
    }
    if let Some(name) = value_capture(
        dialect,
        input,
        target.span,
        &binding_name,
        binding_value_view,
        &reference_spans,
    )
    .first()
    {
        anyhow::bail!(
            "inline-let would capture variable `{name}`: it is free in the binding value but a nested binding in the let body would shadow it, changing the meaning of the code"
        );
    }

    let first_body = &target.children[2];
    let last_body = target
        .children
        .last()
        .expect("body exists after validation");
    Ok(Parts {
        let_span: target.span,
        binding_name,
        binding_value: binding_value_view.span.slice(input).to_owned(),
        body_count: target.children.len() - 2,
        body_span: ByteSpan::new(first_body.span.start(), last_body.span.end()),
        reference_spans,
    })
}

fn vector_let_binding(binding_form: &ExpressionView) -> Result<(String, &ExpressionView)> {
    if binding_form.kind != ExpressionKind::List
        || binding_form.delimiter != Some(Delimiter::Bracket)
    {
        anyhow::bail!("dialect expects vector let bindings: [name value]");
    }
    if binding_form.children.len() != 2 {
        anyhow::bail!("inline-let currently supports exactly one vector binding");
    }
    let name = atom_text(&binding_form.children[0])
        .context("let binding name must be an atom")?
        .to_owned();
    Ok((name, &binding_form.children[1]))
}

fn list_pair_let_binding(binding_form: &ExpressionView) -> Result<(String, &ExpressionView)> {
    if binding_form.kind != ExpressionKind::List || binding_form.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!("dialect expects list-pair let bindings: ((name value))");
    }
    if binding_form.children.len() != 1 {
        anyhow::bail!("inline-let currently supports exactly one list-pair binding");
    }
    let pair = &binding_form.children[0];
    if pair.kind != ExpressionKind::List || pair.delimiter != Some(Delimiter::Paren) {
        anyhow::bail!("let binding must be a (name value) pair");
    }
    if pair.children.len() != 2 {
        anyhow::bail!("let binding pair must contain a name and value");
    }
    let name = atom_text(&pair.children[0])
        .context("let binding name must be an atom")?
        .to_owned();
    Ok((name, &pair.children[1]))
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

fn replace_body_references(
    input: &str,
    body_span: ByteSpan,
    reference_spans: &[ByteSpan],
    replacement: &str,
) -> String {
    let body_start = body_span.start().get();
    let mut output = body_span.slice(input).to_owned();
    let mut spans = reference_spans.to_vec();
    spans.sort_by_key(|span| span.start());
    for span in spans.into_iter().rev() {
        output.replace_range(
            span.start().get() - body_start..span.end().get() - body_start,
            replacement,
        );
    }
    output
}

fn replace_span(input: &str, span: ByteSpan, replacement: &str) -> String {
    let mut output = String::with_capacity(input.len() - span.len() + replacement.len());
    output.push_str(&input[..span.start().get()]);
    output.push_str(replacement);
    output.push_str(&input[span.end().get()..]);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(input: &str, dialect: Dialect, allow_duplicate_evaluation: bool) -> Result<CorePlan> {
        let tree = SyntaxTree::parse(input)?;
        let target = tree.select_path(&Path::from_indexes(vec![0]))?.view();
        super::plan(CoreRequest {
            input,
            dialect,
            path: Some(Path::from_indexes(vec![0])),
            target,
            allow_duplicate_evaluation,
        })
    }

    #[test]
    fn supports_list_and_vector_binding_dialects() {
        let common_lisp = plan("(let ((x 1)) (+ x y))", Dialect::CommonLisp, false).unwrap();
        assert_eq!(common_lisp.rewritten, "(+ 1 y)");
        let clojure = plan("(let [x 1] (+ x y))", Dialect::Clojure, false).unwrap();
        assert_eq!(clojure.rewritten, "(+ 1 y)");
    }

    #[test]
    fn rejects_unused_duplicate_and_capture_cases() {
        assert!(plan("(let ((x (effect))) y)", Dialect::CommonLisp, false).is_err());
        assert!(plan("(let ((x (effect))) (+ x x))", Dialect::CommonLisp, false).is_err());
        assert!(plan("(let ((x (effect))) (+ x x))", Dialect::CommonLisp, true).is_ok());
        assert!(plan("(let ((x y)) (let ((y 2)) x))", Dialect::CommonLisp, false).is_err());
    }
}
