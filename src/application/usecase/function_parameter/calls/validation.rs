use anyhow::{Context, Result};

use crate::application::usecase::function_parameter::list_edit::atom_text;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, SymbolName};

pub(super) fn ensure_matching_function_call(
    view: &ExpressionView,
    function_name: &SymbolName,
    command: &str,
) -> Result<()> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        anyhow::bail!("{command} call selection must be a function call list");
    }
    let head = atom_text(
        view.children
            .first()
            .with_context(|| format!("{command} call must not be empty"))?,
    )
    .with_context(|| format!("{command} call must start with an atom"))?;
    if head != function_name.as_str() {
        anyhow::bail!(
            "{command} call head '{}' does not match selected definition '{}'",
            head,
            function_name
        );
    }

    Ok(())
}
