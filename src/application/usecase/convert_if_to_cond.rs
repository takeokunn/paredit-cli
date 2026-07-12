//! Application facade for converting a selected `if` form into `cond`.

use anyhow::Result;

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::convert_control as domain;
use crate::domain::sexpr::SyntaxTree;

pub use domain::{ConvertIfToCondPlan, ConvertIfToCondRequest};

pub fn plan_convert_if_to_cond(request: ConvertIfToCondRequest<'_>) -> Result<ConvertIfToCondPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    domain::plan_convert_if_to_cond(request)
}
