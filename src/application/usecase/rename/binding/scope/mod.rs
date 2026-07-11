mod reader;
mod special_forms;

use crate::domain::common_lisp::{common_lisp_symbol_reference_eq, is_common_lisp_declaration_form};
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, ReaderPrefix, SymbolName};

use reader::{
    apply_reader_prefix_context, atom_symbol_span, atom_symbol_text, collect_explicit_reader_form,
};

pub(in crate::application::usecase::rename) fn collect_symbol_atom_spans_unshadowed(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    collect_symbol_atom_spans_unshadowed_in_context(
        view,
        symbol,
        output,
        shadowed_scope_count,
        input,
        0,
    );
}

pub(super) fn collect_symbol_atom_spans_unshadowed_in_context(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
    quasiquote_depth: usize,
) {
    let Some(quasiquote_depth) = apply_reader_prefix_context(view, quasiquote_depth) else {
        return;
    };

    if view.kind == ExpressionKind::Atom {
        if view.reader_prefixes.contains(&ReaderPrefix::Function) {
            return;
        }

        if quasiquote_depth == 0
            && atom_symbol_text(view)
                .is_some_and(|text| common_lisp_symbol_reference_eq(text, symbol.as_str()))
        {
            if let Some(span) = atom_symbol_span(view) {
                output.push(span);
            }
        }
        return;
    }

    if view
        .children
        .first()
        .and_then(super::super::selection::atom_text)
        .is_some_and(is_common_lisp_declaration_form)
    {
        return;
    }

    if collect_explicit_reader_form(
        view,
        symbol,
        output,
        shadowed_scope_count,
        input,
        quasiquote_depth,
    ) {
        return;
    }

    if quasiquote_depth > 0 {
        for child in &view.children {
            collect_symbol_atom_spans_unshadowed_in_context(
                child,
                symbol,
                output,
                shadowed_scope_count,
                input,
                quasiquote_depth,
            );
        }
        return;
    }

    if collect_shadow_aware_special_form(view, symbol, output, shadowed_scope_count, input) {
        return;
    }

    for child in &view.children {
        collect_symbol_atom_spans_unshadowed_in_context(
            child,
            symbol,
            output,
            shadowed_scope_count,
            input,
            0,
        );
    }
}

pub(super) fn collect_shadow_aware_special_form(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) -> bool {
    special_forms::collect_shadow_aware_special_form(
        view,
        symbol,
        output,
        shadowed_scope_count,
        input,
    )
}
