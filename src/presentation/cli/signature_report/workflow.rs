use anyhow::{Context, Result, bail};

use crate::application::usecase::signature_report::{
    SignatureReportSource, build_signature_reports, evaluate_signature_report_policy,
};
use crate::domain::sexpr::SyntaxTree;
use crate::presentation::cli::shared::{detect_dialect, read_input};
use crate::presentation::cli::signature_report::args::SignatureReportArgs;
use crate::presentation::cli::signature_report::render::print_signature_report;

pub(in crate::presentation::cli) fn signature_report(args: SignatureReportArgs) -> Result<()> {
    let symbol = args.symbol.as_ref();
    let mut sources = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        sources.push(SignatureReportSource {
            path: file.clone(),
            dialect,
            tree,
        });
    }

    let reports = build_signature_reports(sources, symbol)?;
    let policy = evaluate_signature_report_policy(
        &reports,
        args.fail_on_mismatch,
        args.require_definitions,
        args.require_calls,
    );
    print_signature_report(&reports, symbol, &policy, args.output)?;
    if !policy.passed {
        bail!("signature-report policy failed");
    }
    Ok(())
}
