use anyhow::{Context, Result};

use super::reader::executable_reader_context_at_path;
use crate::domain::dialect::Dialect;
use crate::domain::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::sexpr::{SymbolName, SyntaxTree};

mod candidate;
mod error;
mod safety;
mod selection;
mod types;

use candidate::{SpecializedCandidateContext, add_specialized_candidates, binding_candidates};
pub use error::RenameAtError;
pub use types::{RenameAtNamespace, RenameAtPlan, RenameAtRequest};

pub fn plan_rename_at(request: RenameAtRequest<'_>) -> Result<RenameAtPlan> {
    if request.dialect != Dialect::CommonLisp {
        return Err(RenameAtError::UnsupportedDialect.into());
    }
    if request.at.get() >= request.input.len() || !request.input.is_char_boundary(request.at.get())
    {
        return Err(RenameAtError::InvalidSelection.into());
    }

    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect).map_err(RenameAtError::from)?;
    let selected = tree
        .atom_occurrences()
        .into_iter()
        .find(|occurrence| occurrence.span.contains(request.at))
        .ok_or(RenameAtError::InvalidSelection)?;
    let path = selected.path.clone();
    if !executable_reader_context_at_path(&tree, request.dialect, &path)? {
        return Err(RenameAtError::InertReaderContext.into());
    }
    if selected.text.contains(':') || request.to.as_str().contains(':') {
        return Err(RenameAtError::UnsupportedPackageSyntax.into());
    }
    let from = SymbolName::new(selected.text.clone()).context("selected atom is not a symbol")?;
    let mut candidates = binding_candidates(&tree, request.input, &path, &from, &request.to)?;
    add_specialized_candidates(
        &mut candidates,
        SpecializedCandidateContext {
            input: request.input,
            dialect: request.dialect,
            tree: &tree,
            path: &path,
            selected_span: selected.span,
            from: &from,
            to: &request.to,
        },
    )?;

    let candidate = match candidates.len() {
        0 => return Err(RenameAtError::Unresolved.into()),
        1 => candidates
            .pop()
            .ok_or_else(|| anyhow::anyhow!("one candidate"))?,
        _ => return Err(RenameAtError::Ambiguous.into()),
    };
    SyntaxTree::parse(&candidate.rewritten)
        .context("renamed output is not a valid S-expression document")?;
    Ok(RenameAtPlan {
        dialect: request.dialect,
        namespace: candidate.namespace,
        selection_span: selected.span,
        from,
        to: request.to,
        occurrences: candidate.occurrences,
        changed: candidate.rewritten != request.input,
        rewritten: candidate.rewritten,
    })
}

#[cfg(test)]
mod tests;
