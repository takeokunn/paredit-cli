//! Application facade for converting a non-recursive Common Lisp `labels` into `flet`.

use anyhow::Result;

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::dialect::Dialect;
use crate::domain::local_function_binding as domain;
use crate::domain::sexpr::SyntaxTree;

pub use domain::{ConvertLabelsToFletPlan, ConvertLabelsToFletRequest};

pub fn plan_convert_labels_to_flet(
    request: ConvertLabelsToFletRequest<'_>,
) -> Result<ConvertLabelsToFletPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    if request.dialect == Dialect::CommonLisp {
        reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    }
    domain::plan_convert_labels_to_flet(request)
}
