use anyhow::{Context, Result};

use crate::application::usecase::call_report::build_call_report;
use crate::domain::sexpr::SyntaxTree;
use crate::presentation::cli::call_report::{
    args::CallReportArgs, render::print_call_report, types::CallReportFile,
};
use crate::presentation::cli::shared::{detect_dialect, read_input};

pub(in crate::presentation::cli) fn call_report(args: CallReportArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
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
