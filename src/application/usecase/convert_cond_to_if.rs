//! Application facade for converting a selected `cond` form into nested `if` forms.

use anyhow::Result;

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::convert_control as domain;
use crate::domain::sexpr::SyntaxTree;

pub use domain::{ConvertCondToIfPlan, ConvertCondToIfRequest};

pub fn plan_convert_cond_to_if(request: ConvertCondToIfRequest<'_>) -> Result<ConvertCondToIfPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    domain::plan_convert_cond_to_if(request)
}
