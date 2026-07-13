//! Application facade for splitting sequential `let*` bindings.

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::binding_index::BindingIndex;
use crate::domain::dialect::Dialect;
use crate::domain::let_star_composition::{self, Request as DomainRequest};
use crate::domain::sexpr::{ByteSpan, Path, SyntaxTree};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SplitLetStarRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
    pub binding_index: usize,
}
#[derive(Debug, Clone)]
pub struct SplitLetStarPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub binding_index: usize,
    pub outer_binding_count: usize,
    pub inner_binding_count: usize,
    pub rewritten: String,
    pub changed: bool,
}
pub fn plan_split_let_star(request: SplitLetStarRequest<'_>) -> Result<SplitLetStarPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let binding_index = BindingIndex::new(request.binding_index)?;
    let p = let_star_composition::plan(DomainRequest {
        input: request.input,
        dialect: request.dialect,
        path: request.path.clone(),
        binding_index,
    })?;
    Ok(SplitLetStarPlan {
        dialect: p.dialect,
        path: p.path,
        form_span: p.form_span,
        binding_index: p.binding_index.get(),
        outer_binding_count: p.outer_binding_count,
        inner_binding_count: p.inner_binding_count,
        rewritten: p.rewritten,
        changed: p.changed,
    })
}
