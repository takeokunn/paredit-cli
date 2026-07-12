//! Application safety policy for the Domain `progn` transformation.

use anyhow::{Result, bail};

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::progn as domain;
use crate::domain::sexpr::reader::atom_symbol_text;
use crate::domain::sexpr::{Path, SyntaxTree};

pub use domain::{FlattenPrognPlan, FlattenPrognRequest};

pub fn plan_flatten_progn(request: FlattenPrognRequest<'_>) -> Result<FlattenPrognPlan> {
    if request.path.indexes().len() < 2 {
        bail!("flatten-progn refuses to rewrite a top-level progn");
    }
    let tree = SyntaxTree::parse(request.input)?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    reject_unsafe_context(&tree, &request.path)?;
    domain::plan_flatten_progn(request)
}

fn reject_unsafe_context(tree: &SyntaxTree, path: &Path) -> Result<()> {
    if path.indexes().last().is_some_and(|index| index.get() == 0) {
        bail!("flatten-progn refuses to rewrite an operator position");
    }
    let mut ancestor = path.parent();
    while let Some(ancestor_path) = ancestor {
        if ancestor_path.indexes().is_empty() {
            break;
        }
        let view = tree.select_path(&ancestor_path)?.view();
        if !view.reader_prefixes.is_empty() {
            bail!("flatten-progn refuses to rewrite inside a reader template");
        }
        if view
            .children
            .first()
            .and_then(atom_symbol_text)
            .is_some_and(|head| common_lisp_symbol_reference_eq(head, "declare"))
        {
            bail!("flatten-progn refuses to rewrite inside a declaration");
        }
        ancestor = ancestor_path.parent();
    }
    Ok(())
}
