//! Application facade for converting a capture-free Common Lisp `flet` into `labels`.

use anyhow::Result;

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::dialect::Dialect;
use crate::domain::local_function_binding as domain;
use crate::domain::sexpr::SyntaxTree;

pub use domain::{ConvertFletToLabelsPlan, ConvertFletToLabelsRequest};

pub fn plan_convert_flet_to_labels(
    request: ConvertFletToLabelsRequest<'_>,
) -> Result<ConvertFletToLabelsPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    if request.dialect == Dialect::CommonLisp {
        reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    }
    domain::plan_convert_flet_to_labels(request)
}
