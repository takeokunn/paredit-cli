use crate::domain::common_lisp::{
    common_lisp_operator_head_eq, common_lisp_symbol_name_eq, normalize_common_lisp_operator_head,
};
use crate::domain::sexpr::reader::apply_reader_prefix_context;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, ReaderPrefix, SymbolName};

mod binding_forms;
mod body;
mod lambda_lists;

use binding_forms::collect_shadow_aware_special_form;

pub fn collect_unshadowed_symbol_references(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
) {
    collect_unshadowed_symbol_references_in_context(view, symbol, input, output, 0);
}

pub(super) fn collect_unshadowed_symbol_references_in_context(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
    quasiquote_depth: usize,
) {
    let Some(quasiquote_depth) = apply_reader_prefix_context(view, quasiquote_depth) else {
        return;
    };

    if view.kind == ExpressionKind::Atom {
        collect_atom_reference(view, symbol, output, quasiquote_depth);
        return;
    }

    if collect_explicit_reader_form(view, symbol, input, output, quasiquote_depth) {
        return;
    }

    if quasiquote_depth > 0 {
        collect_quasiquoted_children(view, symbol, input, output, quasiquote_depth);
        return;
    }

    if collect_shadow_aware_special_form(view, symbol, input, output) {
        return;
    }

    for child in &view.children {
        collect_unshadowed_symbol_references_in_context(child, symbol, input, output, 0);
    }
}

fn collect_atom_reference(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    quasiquote_depth: usize,
) {
    if view.reader_prefixes.contains(&ReaderPrefix::Function) {
        return;
    }

    if quasiquote_depth == 0
        && super::syntax::atom_symbol_text(view)
            .is_some_and(|text| common_lisp_symbol_name_eq(text, symbol.as_str()))
    {
        if let Some(span) = super::syntax::atom_symbol_span(view) {
            output.push(span);
        }
    }
}

fn collect_quasiquoted_children(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
    quasiquote_depth: usize,
) {
    for child in &view.children {
        collect_unshadowed_symbol_references_in_context(
            child,
            symbol,
            input,
            output,
            quasiquote_depth,
        );
    }
}

fn collect_explicit_reader_form(
    view: &ExpressionView,
    symbol: &SymbolName,
    input: &str,
    output: &mut Vec<ByteSpan>,
    quasiquote_depth: usize,
) -> bool {
    if view.kind != ExpressionKind::List || view.children.len() < 2 {
        return false;
    }

    let Some(head) = super::syntax::atom_text(&view.children[0]) else {
        return false;
    };

    let normalized_head = normalize_common_lisp_operator_head(head);

    if common_lisp_operator_head_eq(normalized_head, "quote")
        || common_lisp_operator_head_eq(normalized_head, "function")
    {
        return true;
    }

    match normalized_head.to_ascii_lowercase().as_str() {
        "quasiquote" => {
            for child in &view.children[1..] {
                collect_unshadowed_symbol_references_in_context(
                    child,
                    symbol,
                    input,
                    output,
                    quasiquote_depth + 1,
                );
            }
            true
        }
        "unquote" | "unquote-splicing" if quasiquote_depth > 0 => {
            for child in &view.children[1..] {
                collect_unshadowed_symbol_references_in_context(
                    child,
                    symbol,
                    input,
                    output,
                    quasiquote_depth - 1,
                );
            }
            true
        }
        _ => false,
    }
}
