//! Application facade for safely splitting a parallel `let`.

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::dialect::Dialect;
use crate::domain::let_composition::{self, SplitLetRequest as DomainRequest};
use crate::domain::sexpr::{ByteSpan, Path, SyntaxTree};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SplitLetRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
    pub binding_index: usize,
}
#[derive(Debug, Clone)]
pub struct SplitLetPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub binding_index: usize,
    pub outer_binding_count: usize,
    pub inner_binding_count: usize,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_split_let(request: SplitLetRequest<'_>) -> Result<SplitLetPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let plan = let_composition::plan_split_let(DomainRequest {
        input: request.input,
        dialect: request.dialect,
        path: request.path.clone(),
        binding_index: request.binding_index,
    })?;
    Ok(SplitLetPlan {
        dialect: plan.dialect,
        path: plan.path,
        form_span: plan.form_span,
        binding_index: plan
            .binding_index
            .expect("split-let domain plan has a boundary"),
        outer_binding_count: plan.outer_binding_count,
        inner_binding_count: plan.inner_binding_count,
        rewritten: plan.rewritten,
        changed: plan.changed,
    })
}
