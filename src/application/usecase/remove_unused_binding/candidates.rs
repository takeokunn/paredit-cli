use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView};

use super::syntax::atom_text;

#[derive(Debug)]
pub(super) struct LetBindingRemovalCandidate {
    pub(super) index: usize,
    pub(super) name: String,
    pub(super) value_span: ByteSpan,
    pub(super) removal_span: ByteSpan,
}

pub(super) fn let_binding_removal_candidates(
    dialect: Dialect,
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    match dialect {
        Dialect::Clojure | Dialect::Janet | Dialect::Fennel => {
            vector_let_binding_removal_candidates(binding_form)
        }
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Scheme | Dialect::Unknown => {
            list_pair_let_binding_removal_candidates(binding_form)
        }
    }
}

fn vector_let_binding_removal_candidates(
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    if binding_form.kind != ExpressionKind::List
        || binding_form.delimiter != Some(Delimiter::Bracket)
    {
        anyhow::bail!("dialect expects vector let bindings: [name value ...]");
    }
    if binding_form.children.len() % 2 != 0 {
        anyhow::bail!("vector let binding form must contain name/value pairs");
    }

    binding_form
        .children
        .chunks_exact(2)
        .enumerate()
        .map(|(index, pair)| {
            let name = atom_text(&pair[0])
                .context("let binding name must be an atom")?
                .to_owned();
            Ok(LetBindingRemovalCandidate {
                index,
                name,
                value_span: pair[1].span,
                removal_span: ByteSpan::new(pair[0].span.start(), pair[1].span.end()),
            })
        })
        .collect()
}

fn list_pair_let_binding_removal_candidates(
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    if binding_form.kind != ExpressionKind::List || binding_form.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!("dialect expects list-pair let bindings: ((name value) ...)");
    }

    binding_form
        .children
        .iter()
        .enumerate()
        .map(|(index, pair)| {
            if pair.kind != ExpressionKind::List || pair.delimiter != Some(Delimiter::Paren) {
                anyhow::bail!("let binding must be a (name value) pair");
            }
            if pair.children.len() != 2 {
                anyhow::bail!("let binding pair must contain a name and value");
            }
            let name = atom_text(&pair.children[0])
                .context("let binding name must be an atom")?
                .to_owned();
            Ok(LetBindingRemovalCandidate {
                index,
                name,
                value_span: pair.children[1].span,
                removal_span: pair.span,
            })
        })
        .collect()
}
