use anyhow::Result;

use crate::domain::common_lisp::{
    CommonLispBindingRefactorForm, CommonLispOperator, common_lisp_binding_refactor_form_for_head,
    common_lisp_symbol_reference_eq, is_common_lisp_declaration_form,
};
use crate::domain::definition::definition_shape;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, ReaderPrefix, SymbolName, SyntaxTree,
};

use super::super::RenameFunctionOccurrence;
use super::super::binding::collect_shadow_aware_special_form;
use super::super::binding::collect_symbol_atom_spans_unshadowed;
use super::super::binding::parameter_form_binds;
use super::super::reader::atom_symbol_span;
use super::super::reader::{
    apply_reader_prefix_context, explicit_reader_form_kind,
    explicit_reader_function_lambda_body_children,
};
use super::super::selection::atom_text;
use super::shared::{SymbolReferenceSite, is_target_define_symbol_macro};

pub fn collect_define_symbol_macro_reference_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<RenameFunctionOccurrence>> {
    let mut renames = Vec::new();

    for (top_index, _) in tree.root_children().iter().enumerate() {
        let form_path = Path::root_child(top_index);
        let view = tree.select_path(&form_path)?.view();

        if is_target_define_symbol_macro(&view, dialect, from) {
            continue;
        }

        collect_reference_renames_from_view(&view, form_path, dialect, from, to, &mut renames);
    }

    renames.sort_by_key(|rename| rename.span.start());
    renames.dedup_by(|left, right| left.span == right.span);
    Ok(renames)
}

fn collect_reference_renames_from_view(
    view: &ExpressionView,
    path: Path,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let mut reference_spans = Vec::new();
    let mut shadowed_scope_count = 0usize;
    collect_symbol_atom_spans_unshadowed(
        view,
        from,
        &mut reference_spans,
        &mut shadowed_scope_count,
        "",
    );
    collect_symbol_atom_spans_unshadowed_in_reader_function_lambdas(
        view,
        from,
        &mut reference_spans,
        &mut shadowed_scope_count,
        "",
    );
    reference_spans.sort_by_key(|span| span.start());
    reference_spans.dedup();
    if reference_spans.is_empty() {
        return;
    }

    let mut sites = Vec::new();
    collect_symbol_reference_sites(view, path, false, dialect, from, &mut sites);

    for span in reference_spans {
        let Some(site) = sites
            .iter()
            .find(|site| site.span == span && !site.is_head_position)
        else {
            continue;
        };
        renames.push(RenameFunctionOccurrence {
            path: site.path.to_string(),
            span: site.span,
            text: from.as_str().to_owned(),
            replacement: to.as_str().to_owned(),
        });
    }
}

fn collect_symbol_atom_spans_unshadowed_in_reader_function_lambdas(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) {
    collect_symbol_spans_in_context(view, symbol, output, shadowed_scope_count, input, 0);
}

#[allow(clippy::too_many_arguments)]
fn collect_symbol_spans_in_context(
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
            && atom_text(view)
                .is_some_and(|text| common_lisp_symbol_reference_eq(text, symbol.as_str()))
        {
            if let Some(span) = atom_symbol_span(view) {
                output.push(span);
            }
        }
        return;
    }

    if is_common_lisp_declaration_view(view) {
        return;
    }

    if collect_explicit_reader_form_symbol_spans(
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
            collect_symbol_spans_in_context(
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

    if collect_common_lisp_function_definition_symbol_spans(
        view,
        symbol,
        output,
        shadowed_scope_count,
        input,
    ) {
        return;
    }

    if collect_shadow_aware_special_form(view, symbol, output, shadowed_scope_count, input) {
        return;
    }

    for child in &view.children {
        collect_symbol_spans_in_context(child, symbol, output, shadowed_scope_count, input, 0);
    }
}

fn is_common_lisp_declaration_view(view: &ExpressionView) -> bool {
    view.kind == ExpressionKind::List
        && view
            .children
            .first()
            .and_then(atom_text)
            .is_some_and(is_common_lisp_declaration_form)
}

fn collect_common_lisp_function_definition_symbol_spans(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
) -> bool {
    let Some(head) = view.children.first().and_then(atom_text) else {
        return false;
    };

    if common_lisp_binding_refactor_form_for_head(head)
        != Some(CommonLispBindingRefactorForm::FunctionDefinition)
    {
        return false;
    }

    let Some(shape) = definition_shape(Dialect::CommonLisp, view, head) else {
        return false;
    };

    if shape
        .lambda_list(view)
        .is_some_and(|parameter_form| parameter_form_binds(parameter_form, symbol, input))
    {
        *shadowed_scope_count += 1;
        return true;
    }

    if matches!(
        CommonLispOperator::from_head(head),
        Some(CommonLispOperator::DefineSetfExpander | CommonLispOperator::DefineCompilerMacro)
    ) {
        return true;
    }

    for body in shape.body_forms(view) {
        collect_symbol_spans_in_context(body, symbol, output, shadowed_scope_count, input, 0);
    }

    true
}

#[allow(clippy::too_many_arguments)]
fn collect_explicit_reader_form_symbol_spans(
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
    shadowed_scope_count: &mut usize,
    input: &str,
    quasiquote_depth: usize,
) -> bool {
    if view.kind != ExpressionKind::List || view.children.len() < 2 {
        return false;
    }

    let Some(kind_name) = explicit_reader_form_kind(view) else {
        return false;
    };

    match kind_name.as_str() {
        "quote" => true,
        "function" if quasiquote_depth == 0 => {
            if let Some(children) = explicit_reader_function_lambda_body_children(view) {
                for (_, child) in children {
                    collect_symbol_spans_in_context(
                        child,
                        symbol,
                        output,
                        shadowed_scope_count,
                        input,
                        quasiquote_depth,
                    );
                }
            }
            true
        }
        "function" => true,
        "quasiquote" => {
            for child in &view.children[1..] {
                collect_symbol_spans_in_context(
                    child,
                    symbol,
                    output,
                    shadowed_scope_count,
                    input,
                    quasiquote_depth + 1,
                );
            }
            true
        }
        "unquote" | "unquote-splicing" if quasiquote_depth > 0 => {
            for child in &view.children[1..] {
                collect_symbol_spans_in_context(
                    child,
                    symbol,
                    output,
                    shadowed_scope_count,
                    input,
                    quasiquote_depth - 1,
                );
            }
            true
        }
        _ => false,
    }
}

fn collect_symbol_reference_sites(
    view: &ExpressionView,
    path: Path,
    is_head_position: bool,
    dialect: Dialect,
    from: &SymbolName,
    sites: &mut Vec<SymbolReferenceSite>,
) {
    if is_target_define_symbol_macro(view, dialect, from) {
        return;
    }

    if view.kind == ExpressionKind::Atom {
        if let Some(span) = atom_symbol_span(view) {
            sites.push(SymbolReferenceSite {
                span,
                path: path.clone(),
                is_head_position,
            });
        }
    }

    let parent_is_paren_list =
        view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren);

    for (child_index, child) in view.children.iter().enumerate() {
        collect_symbol_reference_sites(
            child,
            path.child(child_index),
            parent_is_paren_list && child_index == 0,
            dialect,
            from,
            sites,
        );
    }
}
