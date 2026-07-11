use anyhow::{Context, Result};

use crate::application::form_shape::duplicate_shape;
use crate::domain::sexpr::{Path, SyntaxTree};

mod rewrite;
#[cfg(test)]
mod tests;
mod types;
mod validation;

use rewrite::rewrite_replace_targets;
pub use types::{ReplaceFormsPlan, ReplaceFormsRequest, ReplaceFormsTarget};
use validation::{
    collect_replace_targets, ensure_same_shape_when_required, original_shape_for_targets,
};

pub fn plan_replace_forms(request: ReplaceFormsRequest<'_>) -> Result<ReplaceFormsPlan> {
    let replacement_tree = SyntaxTree::parse(request.replacement)
        .context("--with must be a valid S-expression document")?;
    anyhow::ensure!(
        replacement_tree.root_children().len() == 1,
        "--with must contain exactly one top-level S-expression"
    );
    let replacement_view = replacement_tree.select_path(&Path::root_child(0))?.view();
    let replacement_shape = duplicate_shape(&replacement_view, true);

    let targets = collect_replace_targets(request.input, request.tree, &request.paths)?;
    let original_shape = original_shape_for_targets(&targets);
    ensure_same_shape_when_required(
        &targets,
        original_shape.as_ref(),
        request.require_same_shape,
    )?;

    let rewritten = rewrite_replace_targets(request.input, &targets, request.replacement);
    SyntaxTree::parse(&rewritten)
        .context("replace-forms output is not a valid S-expression document")?;

    let changed = rewritten != request.input;
    Ok(ReplaceFormsPlan {
        targets,
        replacement: request.replacement.to_owned(),
        replacement_shape,
        require_same_shape: request.require_same_shape,
        original_shape,
        changed,
        rewritten,
    })
}
