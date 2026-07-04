use anyhow::Result;

use crate::application::usecase::function_parameter::FunctionParameterInsert;
use crate::application::usecase::function_parameter::list_edit::{
    atom_text, insertion_edit_for_list_item,
};
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

pub(in crate::application::usecase::function_parameter) fn add_optional_function_parameter_call_edit(
    view: ExpressionView,
    function_name: &SymbolName,
    argument: &str,
    argument_index: usize,
) -> Result<(ByteSpan, String)> {
    ensure_matching_function_call(&view, function_name, "add-function-parameter")?;

    let insertion_item_index = argument_index + 1;
    if insertion_item_index > view.children.len() {
        anyhow::bail!(
            "add-function-parameter call to '{}' at {}..{} does not have {} positional argument(s) before optional argument",
            function_name,
            view.span.start().get(),
            view.span.end().get(),
            argument_index
        );
    }

    insertion_edit_for_list_item(
        &view,
        insertion_item_index,
        argument,
        FunctionParameterInsert::Start,
    )
}

pub(in crate::application::usecase::function_parameter) fn add_keyword_function_parameter_call_edit(
    view: ExpressionView,
    function_name: &SymbolName,
    keyword: &str,
    argument: &str,
    positional_prefix_count: usize,
    insert: FunctionParameterInsert,
) -> Result<(ByteSpan, String)> {
    ensure_matching_function_call(&view, function_name, "add-function-parameter")?;

    let first_keyword_item_index = positional_prefix_count + 1;
    if first_keyword_item_index > view.children.len() {
        anyhow::bail!(
            "add-function-parameter call to '{}' at {}..{} does not have {} positional argument(s) before keyword arguments",
            function_name,
            view.span.start().get(),
            view.span.end().get(),
            positional_prefix_count
        );
    }

    let mut item_index = first_keyword_item_index;
    while item_index < view.children.len() {
        if atom_text(&view.children[item_index]).is_some_and(|text| text == keyword) {
            anyhow::bail!(
                "add-function-parameter call to '{}' at {}..{} already contains keyword argument {}",
                function_name,
                view.span.start().get(),
                view.span.end().get(),
                keyword
            );
        }
        if item_index + 1 >= view.children.len() {
            anyhow::bail!(
                "add-function-parameter call to '{}' at {}..{} has keyword argument without a value",
                function_name,
                view.span.start().get(),
                view.span.end().get()
            );
        }
        item_index += 2;
    }

    let argument_pair = format!("{keyword} {argument}");
    let insertion_index = match insert {
        FunctionParameterInsert::Start => first_keyword_item_index,
        FunctionParameterInsert::End => view.children.len(),
    };
    insertion_edit_for_list_item(
        &view,
        insertion_index,
        &argument_pair,
        FunctionParameterInsert::Start,
    )
}
