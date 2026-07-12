use anyhow::{Result, bail};

use crate::application::usecase::call_graph_report::{
    CallGraphReportSource, build_call_graph_report, evaluate_call_graph_policy,
};
use crate::presentation::cli::call_graph_report::args::CallGraphArgs;
use crate::presentation::cli::call_graph_report::render::print_call_graph_report;
use crate::presentation::cli::shared::read_input_dialect_and_tree;

pub(in crate::presentation::cli) fn call_graph(args: CallGraphArgs) -> Result<()> {
    let symbol = args.symbol.as_ref();
    let mut sources = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let (_, dialect, tree) = read_input_dialect_and_tree(Some(file.clone()), args.dialect)?;

        sources.push(CallGraphReportSource {
            path: file.clone(),
            dialect,
            tree,
        });
    }

    let report = build_call_graph_report(sources, args.include_external, symbol)?;
    let policy = evaluate_call_graph_policy(
        &report.files,
        symbol,
        args.fail_on_inbound_callers,
        args.require_edges,
        args.require_internal_edges,
    );
    print_call_graph_report(
        &report.files,
        &report.nodes_by_name,
        symbol,
        args.include_external,
        &policy,
        args.output,
    )?;
    if !policy.passed {
        bail!("call-graph policy failed");
    }
    Ok(())
}
