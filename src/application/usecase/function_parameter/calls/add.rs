use anyhow::Result;

use crate::application::usecase::function_parameter::FunctionParameterInsert;
use crate::application::usecase::function_parameter::list_edit::insertion_edit_for_list_item;
use crate::domain::sexpr::{ByteSpan, ExpressionView, SymbolName};

use super::validation::ensure_matching_function_call;

pub(in crate::application::usecase::function_parameter) fn add_function_parameter_call_edit(
    view: ExpressionView,
    function_name: &SymbolName,
    argument: &str,
    insert: FunctionParameterInsert,
) -> Result<(ByteSpan, String)> {
    ensure_matching_function_call(&view, function_name, "add-function-parameter")?;

    insertion_edit_for_list_item(&view, 1, argument, insert)
}
