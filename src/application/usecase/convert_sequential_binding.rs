//! Application safety facade for sequential-binding domain plans.

use anyhow::Result;

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::convert_sequential_binding as domain;
use crate::domain::sexpr::SyntaxTree;

pub use domain::{ConvertSequentialBindingPlan, ConvertSequentialBindingRequest};

fn safe(request: &ConvertSequentialBindingRequest<'_>) -> Result<()> {
    let tree = SyntaxTree::parse(request.input)?;
    Ok(reject_common_lisp_reader_conditionals(
        &tree,
        request.dialect,
    )?)
}

pub fn plan_convert_do_star_to_do(
    request: ConvertSequentialBindingRequest<'_>,
) -> Result<ConvertSequentialBindingPlan> {
    safe(&request)?;
    domain::plan_convert_do_star_to_do(request)
}
pub fn plan_convert_prog_star_to_prog(
    request: ConvertSequentialBindingRequest<'_>,
) -> Result<ConvertSequentialBindingPlan> {
    safe(&request)?;
    domain::plan_convert_prog_star_to_prog(request)
}
