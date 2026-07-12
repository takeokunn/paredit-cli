use anyhow::Result;

use super::RenameAtError;
use crate::domain::rename::{RenameFunctionOccurrence, binding_rename_parts};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, ExpressionView, SymbolName};

pub(super) fn ensure_binding_target_is_available(
    view: &ExpressionView,
    from: &SymbolName,
    to: &SymbolName,
    binding_span: ByteSpan,
    input: &str,
) -> Result<()> {
    let Ok(existing) = binding_rename_parts(Dialect::CommonLisp, view, to, input) else {
        return Ok(());
    };
    if existing.binding_span != binding_span && from != to {
        return Err(RenameAtError::NameConflict.into());
    }
    Ok(())
}

pub(super) fn ensure_function_occurrences_are_unqualified(
    definitions: &[RenameFunctionOccurrence],
    calls: &[RenameFunctionOccurrence],
) -> Result<()> {
    if definitions
        .iter()
        .chain(calls)
        .any(|occurrence| occurrence.text.contains(':'))
    {
        return Err(RenameAtError::PackageQualifiedReference.into());
    }
    Ok(())
}
