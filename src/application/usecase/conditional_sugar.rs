//! Application safety facade for conditional-sugar domain plans.

use anyhow::Result;

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::conditional_sugar as domain;
use crate::domain::sexpr::SyntaxTree;

pub use domain::{ConditionalConversionPlan, ConditionalConversionRequest};

fn safe(request: &ConditionalConversionRequest<'_>) -> Result<()> {
    let tree = SyntaxTree::parse(request.input)?;
    Ok(reject_common_lisp_reader_conditionals(
        &tree,
        request.dialect,
    )?)
}

pub fn plan_convert_when_to_if(
    request: ConditionalConversionRequest<'_>,
) -> Result<ConditionalConversionPlan> {
    safe(&request)?;
    domain::plan_convert_when_to_if(request)
}
pub fn plan_convert_unless_to_if(
    request: ConditionalConversionRequest<'_>,
) -> Result<ConditionalConversionPlan> {
    safe(&request)?;
    domain::plan_convert_unless_to_if(request)
}
pub fn plan_convert_if_to_when(
    request: ConditionalConversionRequest<'_>,
) -> Result<ConditionalConversionPlan> {
    safe(&request)?;
    domain::plan_convert_if_to_when(request)
}
pub fn plan_convert_if_to_unless(
    request: ConditionalConversionRequest<'_>,
) -> Result<ConditionalConversionPlan> {
    safe(&request)?;
    domain::plan_convert_if_to_unless(request)
}
