use std::fs;

use anyhow::{Context, Result};

use super::super::{detect_dialect, read_input};
use super::args::RenameBindingArgs;
use super::render::binding::print_rename_binding_plan;
use super::shared::rename_target;
use crate::application::usecase::rename as rename_usecase;

pub(in crate::presentation::cli) fn rename_binding(args: RenameBindingArgs) -> Result<()> {
    if args.write && args.file.is_none() {
        anyhow::bail!("--write requires --file");
    }

    let input = read_input(args.file.clone())?;
    let dialect = detect_dialect(&input, args.dialect);
    let plan = rename_usecase::plan_rename_binding(rename_usecase::RenameBindingRequest {
        input: &input.text,
        dialect,
        target: rename_target(args.path, args.at)?,
        from: args.from,
        to: args.to,
    })?;
    let written = args.write && plan.changed;
    if written {
        let file = args.file.as_ref().context("--write requires --file")?;
        fs::write(file, &plan.rewritten)
            .with_context(|| format!("failed to write {}", file.display()))?;
    }

    print_rename_binding_plan(&plan, written, args.output)
}
