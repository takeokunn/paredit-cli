use anyhow::Result;

use crate::application::usecase::function_parameter::{
    AddFunctionParameterRequest, plan_add_function_parameter,
};
use crate::presentation::cli::{
    detect_dialect, read_input, require_output_file, write_file_with_rollback,
};

use super::args::AddFunctionParameterArgs;
use super::render::add::print_add_function_parameter_plan;

pub(in crate::presentation::cli) fn add_function_parameter(
    args: AddFunctionParameterArgs,
) -> Result<()> {
    if args.write && args.file.is_none() {
        anyhow::bail!("--write requires --file");
    }

    let input = read_input(args.file.clone())?;
    let dialect = detect_dialect(&input, args.dialect);
    let plan = plan_add_function_parameter(AddFunctionParameterRequest {
        input: &input.text,
        dialect,
        definition_path: args.definition_path,
        name: args.name,
        argument: args.argument,
        call_paths: args.call_paths,
        all_calls: args.all_calls,
        insert: args.insert.into_function_parameter_insert(),
        section: args.section.into_function_parameter_section(),
    })?;

    let written = args.write && plan.changed;
    if written {
        let file = require_output_file(input.file.as_ref())?;
        write_file_with_rollback(file.clone(), plan.rewritten.clone())?;
    }

    print_add_function_parameter_plan(&plan, written, args.output)
}
