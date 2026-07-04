use anyhow::Result;

use crate::application::usecase::form_report::types::FormReportRequest;
use crate::application::usecase::form_report::workflow::build_form_report;
use crate::domain::sexpr::SyntaxTree;
use crate::presentation::cli::form_report::{args::FormReportArgs, render::print_form_report};
use crate::presentation::cli::shared::{detect_dialect, read_input, resolve_target};

pub(in crate::presentation::cli) fn form_report(args: FormReportArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    let report = build_form_report(FormReportRequest {
        input: &input.text,
        dialect,
        path: args.path,
        target: selection.view(),
        include_source: args.include_source,
    })?;

    print_form_report(&report, args.output)
}
