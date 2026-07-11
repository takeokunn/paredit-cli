use anyhow::{Context, Result};

use super::super::{detect_dialect, read_input, write_file_with_rollback};
use super::args::RenameInFormArgs;
use super::render::scoped_form::print_rename_in_form_plan;
use super::shared::rename_target;
use crate::application::usecase::rename as rename_usecase;

pub(in crate::presentation::cli) fn rename_in_form(args: RenameInFormArgs) -> Result<()> {
    if args.write && args.file.is_none() {
        anyhow::bail!("--write requires --file");
    }

    let input = read_input(args.file.clone())?;
    let dialect = detect_dialect(&input, args.dialect);
    let plan = rename_usecase::plan_rename_in_form(rename_usecase::RenameInFormRequest {
        input: &input.text,
        dialect,
        target: rename_target(args.path, args.at)?,
        from: args.from,
        to: args.to,
    })?;
    let written = args.write && plan.changed;
    if written {
        let file = args.file.as_ref().context("--write requires --file")?;
        write_file_with_rollback(file.clone(), plan.rewritten.clone())?;
    }

    print_rename_in_form_plan(&plan, written, args.output)
}
