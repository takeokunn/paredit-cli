use anyhow::Result;

use crate::application::usecase::call_report::build_call_report;
use crate::presentation::cli::call_report::{
    args::CallReportArgs, render::print_call_report, types::CallReportFile,
};
use crate::presentation::cli::shared::read_input_dialect_and_tree;

pub(in crate::presentation::cli) fn call_report(args: CallReportArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let (_, dialect, tree) = read_input_dialect_and_tree(Some(file.clone()), args.dialect)?;
        let calls = build_call_report(
            &tree,
            dialect,
            args.symbol.as_ref(),
            args.include_definitions,
        )?;

        reports.push(CallReportFile {
            path: file.clone(),
            dialect,
            calls,
        });
    }

    print_call_report(
        &reports,
        args.symbol.as_ref(),
        args.include_definitions,
        args.output,
    )
}
