use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, SymbolName};

use super::syntax::{atom_text, list_pair_let_binding, vector_let_binding};

#[derive(Debug)]
pub(super) struct InlineLetParts {
    pub(super) let_span: ByteSpan,
    pub(super) binding_name: SymbolName,
    pub(super) binding_value: String,
    pub(super) body_count: usize,
    pub(super) body_span: ByteSpan,
    pub(super) reference_spans: Vec<ByteSpan>,
}

pub(super) fn inline_let_parts(
    dialect: Dialect,
    input: &str,
    target: &ExpressionView,
) -> Result<InlineLetParts> {
    if target.kind != ExpressionKind::List {
        anyhow::bail!("inline-let selection must be a let list");
    }
    if target.children.len() < 3 {
        anyhow::bail!("inline-let requires one binding and at least one body expression");
    }
    let head = atom_text(&target.children[0]).context("inline-let form must start with an atom")?;
    let is_supported_let = dialect.supports_inline_let_refactor_head(head);
    if !is_supported_let {
        anyhow::bail!("inline-let selection must start with let");
    }

    let (binding_name, binding_value_span) = match dialect {
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
        collect_unshadowed_symbol_references(body, &binding_name, input, &mut reference_spans);
    }
    let first_body = &target.children[2];
    let last_body = target
        .children
        .last()
        .context("inline-let expected at least one body expression after validation")?;
    let body_span = ByteSpan::new(first_body.span.start(), last_body.span.end());

    Ok(InlineLetParts {
        let_span: target.span,
        binding_name,
        binding_value: binding_value_span.slice(input).to_owned(),
        body_count: target.children.len() - 2,
        body_span,
        reference_spans,
    })
}
