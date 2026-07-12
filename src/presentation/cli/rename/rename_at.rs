use anyhow::{Context, Result};

use super::super::{detect_dialect, read_input, write_file_with_rollback};
use super::args::RenameAtArgs;
use super::render::at::print_rename_at_plan;
use crate::application::usecase::rename::{RenameAtRequest, plan_rename_at};
use crate::domain::sexpr::ByteOffset;

pub(in crate::presentation::cli) fn rename_at(args: RenameAtArgs) -> Result<()> {
    if args.write && args.file.is_none() {
        anyhow::bail!("--write requires --file");
    }
    let input = read_input(args.file.clone())?;
    let plan = plan_rename_at(RenameAtRequest {
        input: &input.text,
        dialect: detect_dialect(&input, args.dialect),
        at: ByteOffset::new(args.at),
        to: args.to,
    })?;
    let written = args.write && plan.changed;
    if written {
        let file = args.file.as_ref().context("--write requires --file")?;
        write_file_with_rollback(file.clone(), plan.rewritten.clone())?;
    }
    print_rename_at_plan(&plan, written, args.output)
}
