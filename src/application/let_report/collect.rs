use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName};

use super::syntax::{atom_text, list_head, view_at_span};
use super::types::{LetBindingReport, LetFormReport};

#[derive(Debug, Clone)]
struct LetBindingRemovalCandidate {
    index: usize,
    value_span: ByteSpan,
}

pub(super) fn collect_let_reports_from_view(
    dialect: Dialect,
    input: &str,
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    reports: &mut Vec<LetFormReport>,
) -> Result<()> {
    if let Some(report) = analyze_let_form(dialect, input, view, &path_indexes)? {
        reports.push(report);
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_let_reports_from_view(dialect, input, child, child_path, reports)?;
    }

    Ok(())
}

fn analyze_let_form(
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
    if !matches!(head, "let" | "let*") {
        return Ok(None);
    }

    let binding_form = &view.children[1];
    let (binding_style, entries) = let_binding_entries(dialect, binding_form)?;
    let candidates = let_binding_removal_candidates(dialect, binding_form)?;
    let body_count = view.children.len().saturating_sub(2);
    let single_binding = entries.len() == 1;
    let mut bindings = Vec::with_capacity(entries.len());

    for ((name, value_span), candidate) in entries.into_iter().zip(candidates.iter()) {
        let symbol = SymbolName::new(name.clone());
        let reference_count = match &symbol {
            Ok(symbol) => {
                let_binding_reference_spans(view, binding_form, &candidates, candidate, symbol)?
                    .len()
            }
            Err(_) => view.children[2..]
                .iter()
                .map(|body| count_symbol_references(body, &name))
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

        bindings.push(LetBindingReport {
            name,
            value: value_span.slice(input).to_owned(),
            value_span,
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
        inline_supported_by_inline_let: single_binding && body_count > 0,
        bindings,
    }))
}

fn let_binding_entries(
    dialect: Dialect,
    binding_form: &ExpressionView,
) -> Result<(&'static str, Vec<(String, ByteSpan)>)> {
    match dialect {
        Dialect::Clojure | Dialect::Janet | Dialect::Fennel => {
            vector_let_binding_entries(binding_form)
        }
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Scheme | Dialect::Unknown => {
            list_pair_let_binding_entries(binding_form)
        }
    }
}

fn vector_let_binding_entries(
    binding_form: &ExpressionView,
) -> Result<(&'static str, Vec<(String, ByteSpan)>)> {
    if binding_form.kind != ExpressionKind::List
        || binding_form.delimiter != Some(Delimiter::Bracket)
    {
        anyhow::bail!("dialect expects vector let bindings: [name value ...]");
    }
    if binding_form.children.len() % 2 != 0 {
        anyhow::bail!("vector let binding form must contain name/value pairs");
    }

    let entries = binding_form
        .children
        .chunks_exact(2)
        .map(|pair| {
            let name = atom_text(&pair[0])
                .context("let binding name must be an atom")?
                .to_owned();
            Ok((name, pair[1].span))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(("vector", entries))
}

fn list_pair_let_binding_entries(
    binding_form: &ExpressionView,
) -> Result<(&'static str, Vec<(String, ByteSpan)>)> {
    if binding_form.kind != ExpressionKind::List || binding_form.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!("dialect expects list-pair let bindings: ((name value) ...)");
    }

    let entries = binding_form
        .children
        .iter()
        .map(|pair| {
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
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(("list-pair", entries))
}

fn let_binding_removal_candidates(
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
            atom_text(&pair[0]).context("let binding name must be an atom")?;
            Ok(LetBindingRemovalCandidate {
                index,
                value_span: pair[1].span,
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
            atom_text(&pair.children[0]).context("let binding name must be an atom")?;
            Ok(LetBindingRemovalCandidate {
                index,
                value_span: pair.children[1].span,
            })
        })
        .collect()
}

fn let_binding_reference_spans(
    view: &ExpressionView,
    binding_form: &ExpressionView,
    candidates: &[LetBindingRemovalCandidate],
    candidate: &LetBindingRemovalCandidate,
    name: &SymbolName,
) -> Result<Vec<ByteSpan>> {
    let mut reference_spans = Vec::new();
    let sequential_scope = list_head(view).is_some_and(|head| head == "let*")
        || binding_form.delimiter == Some(Delimiter::Bracket);
    if sequential_scope {
        for later in candidates
            .iter()
            .filter(|later| later.index > candidate.index)
        {
            let later_value = view_at_span(binding_form, later.value_span)
                .context("failed to resolve later binding value")?;
            collect_symbol_atom_spans(later_value, name, &mut reference_spans);
        }
    }
    for body in &view.children[2..] {
        collect_symbol_atom_spans(body, name, &mut reference_spans);
    }
    Ok(reference_spans)
}

fn collect_symbol_atom_spans(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
) {
    if atom_text(view).is_some_and(|text| text == symbol.as_str()) {
        output.push(view.span);
        return;
    }
    for child in &view.children {
        collect_symbol_atom_spans(child, symbol, output);
    }
}

fn count_symbol_references(view: &ExpressionView, symbol: &str) -> usize {
    usize::from(atom_text(view).is_some_and(|text| text == symbol))
        + view
            .children
            .iter()
            .map(|child| count_symbol_references(child, symbol))
            .sum::<usize>()
}
