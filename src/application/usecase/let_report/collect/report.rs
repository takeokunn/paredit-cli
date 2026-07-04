use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path, SymbolName};

use super::super::syntax::atom_text;
use super::super::types::{LetBindingReport, LetFormReport};
use super::bindings::let_binding_candidates;
use super::references::{fallback_reference_count, let_binding_reference_count};

pub(super) fn analyze_let_form(
    dialect: Dialect,
    input: &str,
    view: &ExpressionView,
    path_indexes: &[usize],
) -> Result<Option<LetFormReport>> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return Ok(None);
    }
    if view.children.len() < 2 {
        return Ok(None);
    }
    let Some(head) = atom_text(&view.children[0]) else {
        return Ok(None);
    };
    if !matches!(head, "let" | "let*" | "symbol-macrolet") {
        return Ok(None);
    }

    let binding_form = &view.children[1];
    let (binding_style, candidates) = let_binding_candidates(dialect, binding_form)?;
    let body_count = view.children.len().saturating_sub(2);
    let single_binding = candidates.len() == 1;
    let inline_supported_by_inline_let =
        matches!(head, "let" | "let*") && single_binding && body_count > 0;
    let mut bindings = Vec::with_capacity(candidates.len());

    for candidate in &candidates {
        let symbol = SymbolName::new(candidate.name.clone());
        let reference_count = match &symbol {
            Ok(symbol) => let_binding_reference_count(
                input,
                view,
                binding_form,
                &candidates,
                candidate,
                symbol,
            )?,
            Err(_) => view.children[2..]
                .iter()
                .map(|body| fallback_reference_count(body, &candidate.name))
                .sum(),
        };
        let mut risks = Vec::new();
        if !single_binding {
            risks.push("multiple-bindings");
        }
        if symbol.is_err() {
            risks.push("unsupported-binding-name");
        }
        if reference_count == 0 {
            risks.push("unused-binding");
        }
        if reference_count > 1 {
            risks.push("duplicate-evaluation");
        }
        if !inline_supported_by_inline_let {
            risks.push("unsupported-by-inline-let");
        }

        bindings.push(LetBindingReport {
            name: candidate.name.clone(),
            value: candidate.value_span.slice(input).to_owned(),
            value_span: candidate.value_span,
            reference_count,
            can_inline_without_duplication: risks.is_empty(),
            risks,
        });
    }

    Ok(Some(LetFormReport {
        path: Path::from_indexes(path_indexes.to_vec()),
        form: head.to_owned(),
        span: view.span,
        binding_style,
        body_count,
        inline_supported_by_inline_let,
        bindings,
    }))
}
