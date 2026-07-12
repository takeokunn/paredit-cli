use super::*;
use crate::domain::sexpr::ExpressionPath;

fn request<'a>(
    input: &'a str,
    name: &str,
    argument: &str,
    call_paths: Vec<ExpressionPath>,
    all_calls: bool,
    insert: FunctionParameterInsert,
    section: FunctionParameterSection,
) -> AddFunctionParameterRequest<'a> {
    AddFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol(name),
        argument: argument.to_owned(),
        call_paths,
        all_calls,
        insert,
        section,
    }
}

mod all_calls;
mod rejects;
