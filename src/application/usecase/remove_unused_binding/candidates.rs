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

pub(super) fn macrolet_binding_removal_candidates(
    dialect: Dialect,
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    match dialect {
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Scheme | Dialect::Unknown => {
            list_pair_macrolet_binding_removal_candidates(binding_form)
        }
        _ => anyhow::bail!("remove-unused-binding only supports macrolet in Common Lisp"),
    }
}

pub(super) fn local_callable_binding_removal_candidates(
    dialect: Dialect,
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    match dialect {
        Dialect::CommonLisp | Dialect::Unknown => {
            list_pair_local_callable_binding_removal_candidates(binding_form)
        }
        _ => anyhow::bail!("remove-unused-binding only supports flet and labels in Common Lisp"),
    }
}

pub(super) fn with_slots_binding_removal_candidates(
    dialect: Dialect,
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    match dialect {
        Dialect::CommonLisp | Dialect::Unknown => {
            list_pair_with_slots_binding_removal_candidates(binding_form)
        }
        _ => anyhow::bail!("remove-unused-binding only supports with-slots in Common Lisp"),
    }
}

pub(super) fn with_accessors_binding_removal_candidates(
    dialect: Dialect,
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    match dialect {
        Dialect::CommonLisp | Dialect::Unknown => {
            list_pair_with_accessors_binding_removal_candidates(binding_form)
        }
        _ => anyhow::bail!("remove-unused-binding only supports with-accessors in Common Lisp"),
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

fn list_pair_with_slots_binding_removal_candidates(
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    if binding_form.kind != ExpressionKind::List || binding_form.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!("dialect expects with-slots bindings: (slot-or-pair ...)");
    }

    binding_form
        .children
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            if spec.kind == ExpressionKind::Atom {
                let name = atom_text(spec)
                    .context("with-slots bare binding name must be an atom")?
                    .to_owned();
                return Ok(LetBindingRemovalCandidate {
                    index,
                    name,
                    value_span: spec.span,
                    removal_span: spec.span,
                });
            }
            if spec.kind != ExpressionKind::List || spec.delimiter != Some(Delimiter::Paren) {
                anyhow::bail!("with-slots binding must be a slot name or (name slot-name) pair");
            }
            if spec.children.len() != 2 {
                anyhow::bail!("with-slots binding pair must contain a name and slot name");
            }
            let name = atom_text(&spec.children[0])
                .context("with-slots binding name must be an atom")?
                .to_owned();
            Ok(LetBindingRemovalCandidate {
                index,
                name,
                value_span: spec.children[1].span,
                removal_span: spec.span,
            })
        })
        .collect()
}

fn list_pair_with_accessors_binding_removal_candidates(
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    if binding_form.kind != ExpressionKind::List || binding_form.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!("dialect expects with-accessors bindings: ((name accessor) ...)");
    }

    binding_form
        .children
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            if spec.kind != ExpressionKind::List || spec.delimiter != Some(Delimiter::Paren) {
                anyhow::bail!("with-accessors binding must be a (name accessor) pair");
            }
            if spec.children.len() != 2 {
                anyhow::bail!("with-accessors binding pair must contain a name and accessor");
            }
            let name = atom_text(&spec.children[0])
                .context("with-accessors binding name must be an atom")?
                .to_owned();
            Ok(LetBindingRemovalCandidate {
                index,
                name,
                value_span: spec.children[1].span,
                removal_span: spec.span,
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

fn list_pair_macrolet_binding_removal_candidates(
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    if binding_form.kind != ExpressionKind::List || binding_form.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!(
            "dialect expects list-pair macrolet bindings: ((name lambda-list form*) ...)"
        );
    }

    binding_form
        .children
        .iter()
        .enumerate()
        .map(|(index, pair)| {
            if pair.kind != ExpressionKind::List || pair.delimiter != Some(Delimiter::Paren) {
                anyhow::bail!("macrolet binding must be a (name lambda-list form*) list");
            }
            if pair.children.len() < 2 {
                anyhow::bail!("macrolet binding must contain a name and macro expander body");
            }
            let name = atom_text(&pair.children[0])
                .context("macrolet binding name must be an atom")?
                .to_owned();
            let value_start = pair.children[1].span.start();
            let value_end = pair
                .children
                .last()
                .expect("validated macrolet binding has at least two children")
                .span
                .end();
            Ok(LetBindingRemovalCandidate {
                index,
                name,
                value_span: ByteSpan::new(value_start, value_end),
                removal_span: pair.span,
            })
        })
        .collect()
}

fn list_pair_local_callable_binding_removal_candidates(
    binding_form: &ExpressionView,
) -> Result<Vec<LetBindingRemovalCandidate>> {
    if binding_form.kind != ExpressionKind::List || binding_form.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!(
            "dialect expects list-pair local callable bindings: ((name lambda-list form*) ...)"
        );
    }

    binding_form
        .children
        .iter()
        .enumerate()
        .map(|(index, pair)| {
            if pair.kind != ExpressionKind::List || pair.delimiter != Some(Delimiter::Paren) {
                anyhow::bail!("local callable binding must be a (name lambda-list form*) list");
            }
            if pair.children.len() < 2 {
                anyhow::bail!("local callable binding must contain a name, lambda list, and body");
            }
            let name = atom_text(&pair.children[0])
                .context("local callable binding name must be an atom")?
                .to_owned();
            let value_start = pair.children[1].span.start();
            let value_end = pair
                .children
                .last()
                .expect("validated local callable binding has at least two children")
                .span
                .end();
            Ok(LetBindingRemovalCandidate {
                index,
                name,
                value_span: ByteSpan::new(value_start, value_end),
                removal_span: pair.span,
            })
        })
        .collect()
}
