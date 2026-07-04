use std::fs;

use anyhow::{Context, Result};

use crate::application::usecase::function_parameter::{
    RemoveFunctionParameterRequest, plan_remove_function_parameter,
};
use crate::presentation::cli::{detect_dialect, read_input};

use super::args::RemoveFunctionParameterArgs;
use super::render::print_remove_function_parameter_plan;

pub(in crate::presentation::cli) fn remove_function_parameter(
    args: RemoveFunctionParameterArgs,
) -> Result<()> {
    if args.write && args.file.is_none() {
        anyhow::bail!("--write requires --file");
    }

    let input = read_input(args.file.clone())?;
    let dialect = detect_dialect(&input, args.dialect);
    let plan = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input: &input.text,
        dialect,
        definition_path: args.definition_path,
        name: args.name,
        call_paths: args.call_paths,
        all_calls: args.all_calls,
        allow_missing_argument: args.allow_missing_argument,
    })?;

    let written = args.write && plan.changed;
    if written {
        let file = input
            .file
            .as_ref()
            .expect("--write was validated to require --file");
        fs::write(file, &plan.rewritten)
            .with_context(|| format!("failed to write {}", file.display()))?;
    }

    print_remove_function_parameter_plan(&plan, written, args.output)
}
