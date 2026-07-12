use crate::domain::rename::function::target::{
    CallableNameTarget, callable_name_target,
};
use crate::domain::sexpr::ExpressionView;

use super::RenameFunctionOccurrence;
use super::scope::{
    LocalCallableRenameKind, MacroletRenameScope, allows_function_reference_rename,
};
pub(super) use crate::domain::rename::reader::{atom_symbol_span, atom_symbol_text};
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::sexpr::{Path, ReaderPrefix, SymbolName};

#[expect(
    clippy::too_many_arguments,
    reason = "reader designator handling needs rename kind, scope, and accumulator state"
)]
pub(super) fn collect_local_function_designator_renames(
    view: &ExpressionView,
    path: &Path,
    from: &SymbolName,
    to: &SymbolName,
    kind: LocalCallableRenameKind,
    scope: MacroletRenameScope,
    quasiquote_depth: usize,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    if kind != LocalCallableRenameKind::Function || quasiquote_depth > 0 {
        return false;
    }

    if view.reader_prefixes.contains(&ReaderPrefix::Function) {
        if let Some(target) = callable_name_target(view, path) {
            if !allows_function_reference_rename(scope, target.text) {
                return false;
            }
            return push_callable_target_rename_if_match(target, from, to, renames);
        }
    }

    false
}

pub(super) fn push_callable_target_rename_if_match(
    target: CallableNameTarget<'_>,
    from: &SymbolName,
    to: &SymbolName,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    if !common_lisp_symbol_reference_eq(target.text, from.as_str()) {
        return false;
    }

    renames.push(RenameFunctionOccurrence {
        path: target.path.to_string(),
        span: target.span,
        text: target.text.to_owned(),
        replacement: to.as_str().to_owned(),
    });
    true
}

pub(super) fn push_atom_rename_if_match(
    view: &ExpressionView,
    path: &Path,
    from: &SymbolName,
    to: &SymbolName,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> bool {
    let Some(text) = atom_symbol_text(view) else {
        return false;
    };
    if !common_lisp_symbol_reference_eq(text, from.as_str()) {
        return false;
    }

    renames.push(RenameFunctionOccurrence {
        path: path.to_string(),
        span: atom_symbol_span(view).unwrap_or(view.span),
        text: text.to_owned(),
        replacement: to.as_str().to_owned(),
    });
    true
}
