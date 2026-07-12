//! Application facade for converting independent `let*` into `let`.

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::let_binding as domain;
use crate::domain::sexpr::SyntaxTree;
use anyhow::Result;

pub use domain::{ConvertLetStarToLetPlan, ConvertLetStarToLetRequest};

pub fn plan_convert_let_star_to_let(
    request: ConvertLetStarToLetRequest<'_>,
) -> Result<ConvertLetStarToLetPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    domain::plan_convert_let_star_to_let(request)
}
