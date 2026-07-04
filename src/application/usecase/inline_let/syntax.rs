use anyhow::{Context, Result};

use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView};

pub(super) fn vector_let_binding(binding_form: &ExpressionView) -> Result<(String, ByteSpan)> {
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
    Ok((name, binding_form.children[1].span))
}

pub(super) fn list_pair_let_binding(binding_form: &ExpressionView) -> Result<(String, ByteSpan)> {
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
    Ok((name, pair.children[1].span))
}

pub(super) fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
