use anyhow::Result;

use crate::application::usecase::rename::{self as rename_usecase, RenameTarget};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SymbolName, SyntaxTree};

pub(super) fn rename_target(path: Option<Path>, at: Option<usize>) -> Result<RenameTarget> {
    match (path, at) {
        (Some(path), None) => Ok(RenameTarget::Path(path)),
        (None, Some(offset)) => Ok(RenameTarget::Offset(offset)),
        (None, None) => anyhow::bail!("target required: pass --path or --at"),
        (Some(_), Some(_)) => anyhow::bail!("pass only one of --path or --at"),
    }
}

pub(in crate::presentation::cli) fn collect_callable_definition_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<rename_usecase::RenameFunctionOccurrence>> {
    rename_usecase::collect_callable_definition_renames(tree, dialect, from, to)
}

pub(in crate::presentation::cli) fn collect_function_call_head_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<rename_usecase::RenameFunctionOccurrence>> {
    rename_usecase::collect_function_call_head_renames(tree, dialect, from, to)
}
