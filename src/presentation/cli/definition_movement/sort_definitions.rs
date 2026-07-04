use std::fs;

use anyhow::{Context, Result};

use crate::application::usecase::sort_definitions::{
    SortDefinitionsRequest, plan_sort_definitions,
};

use super::super::shared::{detect_dialect, read_input};
use super::args::SortDefinitionsArgs;
use super::render::sort_definitions::print_sort_definitions_plan;

pub(in crate::presentation::cli) fn sort_definitions(args: SortDefinitionsArgs) -> Result<()> {
    let input = read_input(Some(args.file.clone()))?;
    let dialect = detect_dialect(&input, args.dialect);
    let plan = plan_sort_definitions(SortDefinitionsRequest {
        file: args.file.clone(),
        input: &input.text,
        dialect,
        strategy: args.order.into(),
        write: args.write,
    })?;
    if plan.written {
        fs::write(&args.file, &plan.rewritten)
            .with_context(|| format!("failed to write {}", args.file.display()))?;
    }
    print_sort_definitions_plan(&plan, args.output)
}
