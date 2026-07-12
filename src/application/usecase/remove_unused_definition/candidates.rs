use anyhow::{Context, Result};

use crate::application::usecase::callable_scope::{
    LocalCallableName, common_lisp_local_callable_form, is_local_callable_bound,
    local_callable_binding_body_scope, local_callable_body_scope,
};
use crate::application::usecase::remove_unused_definition::types::{
    RemoveUnusedDefinitionInputFile, UnusedDefinitionDefinition,
};
use crate::domain::common_lisp::{
    CommonLispLocalCallableForm, CommonLispPackageDeclarationForm, common_lisp_operator_head_eq,
    common_lisp_symbol_reference_eq,
};
use crate::domain::dialect::Dialect;
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::reader::{atom_symbol_text, atom_text};
use crate::domain::sexpr::{
    ByteSpan, ExpressionKind, ExpressionView, ReaderPrefix, SymbolName, SyntaxTree,
};

#[derive(Debug)]
pub(super) struct UnusedDefinitionItem {
    pub(super) definition: UnusedDefinitionDefinition,
    pub(super) references: Vec<DefinitionReference>,
}

#[derive(Debug)]
pub(super) struct UnusedDefinitionFile {
    pub(super) definitions: Vec<UnusedDefinitionItem>,
}

#[derive(Debug)]
pub(super) struct DefinitionReference;

pub(super) fn collect_unused_definition_candidates(
    files: &[RemoveUnusedDefinitionInputFile],
) -> Result<Vec<UnusedDefinitionFile>> {
    let parsed_files = files
        .iter()
        .map(|file| -> Result<_> {
            let tree = SyntaxTree::parse(&file.text)
                .with_context(|| format!("failed to parse {}", file.path.display()))?;
            Ok((file, tree.root_view()))
        })
        .collect::<Result<Vec<_>>>()?;

    let package_form_spans: Vec<Vec<ByteSpan>> = parsed_files
        .iter()
        .map(|(file, view)| {
            let mut spans = Vec::new();
            collect_package_form_spans(file.dialect, view, &mut spans);
            spans
        })
        .collect();

    files
        .iter()
        .enumerate()
        .map(|(file_index, file)| -> Result<_> {
            let named_definitions = file
                .definitions
                .iter()
                .filter_map(|definition| {
                    let name = definition.name.as_ref()?;
                    Some((definition, name))
                })
                // A category outside `is_bulk_removable` (`System` for
                // `asdf:defsystem`'s string-literal system name `"cl-cli"`,
                // ...) is expected to sometimes report a name that isn't a
                // valid bare symbol, and such a name can never be searched
                // for as a value/function-namespace reference anyway — skip
                // it rather than aborting the whole command, matching how
                // `definition_report::references` (`unused-definition-
                // report`) already treats the same case via `.ok()?`. A
                // bulk-removable category (`Function`, `Macro`, ...)
                // reporting an invalid name instead indicates corrupted
                // upstream metadata, so that case still surfaces loudly.
                .filter_map(|(definition, name)| match SymbolName::new(name.clone()) {
                    Ok(symbol) => Some(Ok((definition, symbol))),
                    Err(_) if !definition.category.is_bulk_removable() => None,
                    Err(error) => Some(Err(error.context(format!(
                        "remove-unused-definition found invalid symbol '{name}' in {}",
                        file.path.display()
                    )))),
                })
                .collect::<Result<Vec<_>>>()?;

            let definitions = named_definitions
                .into_iter()
                .map(|(definition, symbol)| {
                    let references = files
                        .iter()
                        .enumerate()
                        .flat_map(|(other_index, other)| {
                            let (_, other_view) = &parsed_files[other_index];
                            let mut spans = Vec::new();
                            collect_unshadowed_symbol_references(
                                other.dialect,
                                other_view,
                                &symbol,
                                &other.text,
                                &mut spans,
                            );
                            // Scope-aware value-namespace collection above
                            // intentionally treats function-namespace
                            // designators such as `#'name` and `(function
                            // name)` as invisible. Supplement it with a
                            // callable-namespace traversal that understands
                            // local callable shadowing.
                            collect_function_quote_references(
                                other.dialect,
                                other_view,
                                &symbol,
                                &mut spans,
                            );
                            collect_quoted_data_references(
                                other.dialect,
                                other_view,
                                &symbol,
                                &mut spans,
                            );
                            let package_spans = &package_form_spans[other_index];
                            spans.retain(|span| {
                                !package_spans
                                    .iter()
                                    .any(|package| span_contains(*package, *span))
                            });
                            spans
                                .into_iter()
                                .filter(move |span| {
                                    !(other_index == file_index
                                        && span_contains(definition.span, *span))
                                })
                                .map(|_span| DefinitionReference)
                        })
                        .collect();

                    UnusedDefinitionItem {
                        definition: definition.clone(),
                        references,
                    }
                })
                .collect::<Vec<_>>();

            Ok(UnusedDefinitionFile { definitions })
        })
        .collect()
}

fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start().get() <= inner.start().get() && inner.end().get() <= outer.end().get()
}

/// Collects the span of every `defpackage` form. Symbols named inside one —
/// the members of its `:export`/`:import-from`/`:shadow` clauses and the
/// package name itself — are package-system *metadata*, not live value- or
/// function-namespace references. An occurrence there (`#:name` in an
/// `:export` clause) must not count as a use that keeps a definition
/// reachable, otherwise every exported symbol would look referenced and the
/// `exported-definition` skip could never fire.
///
/// `in-package` is deliberately excluded: `(in-package #:demo)` is a genuine
/// reference to the package `demo`, and counting it keeps the matching
/// `defpackage` from being flagged as an unused package definition.
///
/// `pub(crate)`: also used by `definition_report::references` so
/// `unused-definition-report` and `remove-unused-definitions` agree on what
/// counts as a reference instead of drifting apart.
pub(crate) fn collect_package_form_spans(
    dialect: Dialect,
    view: &ExpressionView,
    output: &mut Vec<ByteSpan>,
) {
    if let Some(head) = list_head(view) {
        if dialect.common_lisp_package_declaration_form_for_head(head)
            == Some(CommonLispPackageDeclarationForm::Defpackage)
        {
            output.push(view.span);
            return;
        }
    }

    for child in &view.children {
        collect_package_form_spans(dialect, child, output);
    }
}

/// Collects every function-namespace designator matching `symbol`,
/// appending their spans to `output` while respecting local callable
/// shadowing from `flet`, `labels`, `macrolet`, and `compiler-macrolet`.
///
/// `pub(crate)`: also used by `definition_report::references` so
/// `unused-definition-report` and `remove-unused-definitions` agree on what
/// counts as a reference instead of drifting apart.
pub(crate) fn collect_function_quote_references(
    dialect: Dialect,
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
) {
    collect_function_quote_references_from_view(dialect, view, symbol, &[], output);
}

fn collect_function_quote_references_from_view(
    dialect: Dialect,
    view: &ExpressionView,
    symbol: &SymbolName,
    local_callables: &[LocalCallableName],
    output: &mut Vec<ByteSpan>,
) {
    if let Some(head) = list_head(view) {
        if let Some(form) = common_lisp_local_callable_form(dialect, head) {
            collect_local_callable_function_quote_references(
                dialect,
                view,
                symbol,
                local_callables,
                form,
                output,
            );
            return;
        }
    }

    if let Some(target) = callable_reference_target(view) {
        if callable_reference_matches(dialect, target, symbol)
            && !is_local_callable_bound(local_callables, symbol.as_str())
        {
            output.push(target.span);
            return;
        }
    }

    for child in &view.children {
        collect_function_quote_references_from_view(
            dialect,
            child,
            symbol,
            local_callables,
            output,
        );
    }
}

fn collect_local_callable_function_quote_references(
    dialect: Dialect,
    view: &ExpressionView,
    symbol: &SymbolName,
    local_callables: &[LocalCallableName],
    form: CommonLispLocalCallableForm,
    output: &mut Vec<ByteSpan>,
) {
    let body_scope = local_callable_body_scope(local_callables, view);

    if let Some(bindings) = view.children.get(1) {
        let binding_body_scope =
            local_callable_binding_body_scope(form, local_callables, &body_scope);
        for binding in &bindings.children {
            for child in binding.children.iter().skip(2) {
                collect_function_quote_references_from_view(
                    dialect,
                    child,
                    symbol,
                    binding_body_scope,
                    output,
                );
            }
        }
    }

    for child in view.children.iter().skip(2) {
        collect_function_quote_references_from_view(dialect, child, symbol, &body_scope, output);
    }
}

fn callable_reference_target(view: &ExpressionView) -> Option<&ExpressionView> {
    if view.reader_prefixes.contains(&ReaderPrefix::Function) {
        return Some(view);
    }

    let target = callable_accessor_target(view)?;
    Some(target)
}

fn callable_accessor_target(view: &ExpressionView) -> Option<&ExpressionView> {
    (view.kind == ExpressionKind::List).then_some(())?;
    let head = atom_text(view.children.first()?)?;
    let target = view.children.get(1)?;

    matches!(
        common_lisp_callable_accessor_kind(head),
        Some(CallableAccessorKind::Function | CallableAccessorKind::QuotedFunction)
    )
    .then_some(target)
}

fn callable_reference_matches(
    dialect: Dialect,
    candidate: &ExpressionView,
    symbol: &SymbolName,
) -> bool {
    atom_symbol_text(candidate)
        .is_some_and(|text| function_quote_symbol_matches(dialect, text, symbol.as_str()))
        || setf_callable_name_view(candidate).is_some_and(|name| {
            atom_symbol_text(name)
                .is_some_and(|text| function_quote_symbol_matches(dialect, text, symbol.as_str()))
        })
}

fn setf_callable_name_view(view: &ExpressionView) -> Option<&ExpressionView> {
    (view.kind == ExpressionKind::List).then_some(())?;
    let head = view.children.first().and_then(atom_text)?;
    common_lisp_operator_head_eq(head, "setf").then_some(())?;
    view.children
        .get(1)
        .filter(|name| name.kind == ExpressionKind::Atom)
}

fn common_lisp_callable_accessor_kind(head: &str) -> Option<CallableAccessorKind> {
    if common_lisp_operator_head_eq(head, "function") {
        return Some(CallableAccessorKind::Function);
    }

    if matches!(
        head,
        "macro-function" | "compiler-macro-function" | "symbol-function" | "fdefinition"
    ) || common_lisp_operator_head_eq(head, "macro-function")
        || common_lisp_operator_head_eq(head, "compiler-macro-function")
        || common_lisp_operator_head_eq(head, "symbol-function")
        || common_lisp_operator_head_eq(head, "fdefinition")
    {
        return Some(CallableAccessorKind::QuotedFunction);
    }

    None
}

#[derive(Clone, Copy)]
enum CallableAccessorKind {
    Function,
    QuotedFunction,
}

fn list_head(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::List)
        .then_some(view.children.first())
        .flatten()
        .and_then(atom_text)
}

fn function_quote_symbol_matches(dialect: Dialect, candidate: &str, symbol: &str) -> bool {
    match dialect {
        Dialect::CommonLisp => common_lisp_symbol_reference_eq(candidate, symbol),
        _ => candidate == symbol,
    }
}

/// Collects every bare atom matching `symbol` found anywhere inside a
/// plain-quoted region (a `Quote` reader prefix, e.g. `'((key . command)
/// ...)` dispatch tables and alists) or a quasiquoted region (a
/// `` Quasiquote `` reader prefix, e.g. `` `(,slot-validator ,slot) ``
/// code-generation templates built by macros). Scope-aware reference
/// collection treats both as fully opaque data and does not look inside them
/// at all, which makes it blind to two related idioms: storing a
/// function/variable name as a bare symbol inside a quoted list literal
/// (keymap tables, dispatch alists, `featurep`/`fboundp` argument lists), and
/// naming a function as the literal head of a quasiquoted form a macro
/// builds up to splice into its expansion (a near-universal pattern for
/// macros that assemble generated code piecemeal, e.g. `(push
/// `` `(validator-fn ,arg) `` forms)`). A quasiquoted form's unquoted
/// (`` , ``/`` ,@ ``) sub-expressions are ordinary evaluated references too,
/// so no distinction is needed between the literal and unquoted portions.
/// Unlike scope-aware collection, this needs no shadowing awareness: neither
/// region can introduce a lexical binding, so any bare atom matching
/// `symbol` inside one is unambiguous evidence the definition is reachable,
/// at the cost of occasionally counting an unrelated same-named symbol that
/// only appears as incidental data.
///
/// `pub(crate)`: also used by `definition_report::references` so
/// `unused-definition-report` and `remove-unused-definitions` agree on what
/// counts as a reference instead of drifting apart.
pub(crate) fn collect_quoted_data_references(
    dialect: Dialect,
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
) {
    if view
        .reader_prefixes
        .iter()
        .any(|prefix| matches!(prefix, ReaderPrefix::Quote | ReaderPrefix::Quasiquote))
    {
        collect_atoms_in_quoted_region(dialect, view, symbol, output);
        return;
    }

    if callable_accessor_target(view).is_some() {
        for (index, child) in view.children.iter().enumerate() {
            if index == 1 {
                continue;
            }
            collect_quoted_data_references(dialect, child, symbol, output);
        }
        return;
    }

    for child in &view.children {
        collect_quoted_data_references(dialect, child, symbol, output);
    }
}

fn collect_atoms_in_quoted_region(
    dialect: Dialect,
    view: &ExpressionView,
    symbol: &SymbolName,
    output: &mut Vec<ByteSpan>,
) {
    if view.kind == ExpressionKind::Atom {
        if atom_symbol_text(view)
            .is_some_and(|text| function_quote_symbol_matches(dialect, text, symbol.as_str()))
        {
            output.push(view.span);
        }
        return;
    }

    for child in &view.children {
        collect_atoms_in_quoted_region(dialect, child, symbol, output);
    }
}
