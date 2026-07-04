use std::fs;

use anyhow::{Context, Result};

use super::super::{detect_dialect, read_input};
use super::args::RenameFunctionArgs;
use super::render::function::print_rename_function_report;
use super::types::{PendingRenameFunctionFile, RenameFunctionFileReport};
use crate::application::usecase::rename as rename_usecase;

pub(in crate::presentation::cli) fn rename_function(args: RenameFunctionArgs) -> Result<()> {
    let mut pending = Vec::with_capacity(args.files.len());
    let mut definition_count = 0usize;

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let plan = rename_usecase::plan_rename_function(rename_usecase::RenameFunctionRequest {
            input: &input.text,
            dialect,
            from: args.from.clone(),
            to: args.to.clone(),
        })
        .with_context(|| format!("failed to plan rename-function for {}", file.display()))?;
        let definitions = plan.definitions;
        let calls = plan.calls;
        definition_count += definitions.len();
        pending.push(PendingRenameFunctionFile {
            path: file.clone(),
            dialect: plan.dialect,
            definitions,
            calls,
            rewritten: plan.rewritten,
            changed: plan.changed,
        });
    }

    if definition_count == 0 {
        anyhow::bail!("rename-function requires at least one matching callable definition");
    }

    let mut reports = Vec::with_capacity(pending.len());
    for file in pending {
        let written = args.write && file.changed;
        if written {
            fs::write(&file.path, &file.rewritten)
                .with_context(|| format!("failed to write {}", file.path.display()))?;
        }
        reports.push(RenameFunctionFileReport {
            path: file.path,
            dialect: file.dialect,
            definitions: file.definitions,
            calls: file.calls,
            changed: file.changed,
            written,
            rewritten: file.rewritten,
        });
    }

    print_rename_function_report(&reports, &args.from, &args.to, args.write, args.output)
}
