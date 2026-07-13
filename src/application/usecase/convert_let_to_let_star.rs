//! Application facade for converting dependency-free `let` into `let*`.

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::let_binding as domain;
use crate::domain::sexpr::SyntaxTree;
use anyhow::Result;

pub use domain::{ConvertLetToLetStarPlan, ConvertLetToLetStarRequest};

pub fn plan_convert_let_to_let_star(
    request: ConvertLetToLetStarRequest<'_>,
) -> Result<ConvertLetToLetStarPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    domain::plan_convert_let_to_let_star(request)
}
