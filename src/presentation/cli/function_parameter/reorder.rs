use std::fs;

use anyhow::{Context, Result};

use crate::application::usecase::function_parameter::{
    plan_reorder_function_parameters, ReorderFunctionParametersRequest,
};
use crate::presentation::cli::{detect_dialect, read_input};

use super::args::ReorderFunctionParametersArgs;
use super::render::reorder::print_reorder_function_parameters_plan;

pub(in crate::presentation::cli) fn reorder_function_parameters(
    args: ReorderFunctionParametersArgs,
) -> Result<()> {
    if args.write && args.file.is_none() {
        anyhow::bail!("--write requires --file");
    }

    let input = read_input(args.file.clone())?;
    let dialect = detect_dialect(&input, args.dialect);
    let plan = plan_reorder_function_parameters(ReorderFunctionParametersRequest {
        input: &input.text,
        dialect,
        definition_path: args.definition_path,
        parameter_order: args.parameter_order,
        call_paths: args.call_paths,
        all_calls: args.all_calls,
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

    print_reorder_function_parameters_plan(&plan, written, args.output)
}
