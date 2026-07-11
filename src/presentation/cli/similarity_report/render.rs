use anyhow::Result;
use serde_json::json;

use crate::application::usecase::similarity_report::{SimilarityFormReport, SimilarityReport};
use crate::domain::dialect::Dialect;
use crate::infrastructure::workspace::WorkspaceDiscovery;

use super::super::OutputFormat;
use super::args::SimilarityReportArgs;

pub(in crate::presentation::cli) fn print_similarity_report(
    report: &SimilarityReport,
    discovery: &WorkspaceDiscovery,
    args: &SimilarityReportArgs,
) -> Result<()> {
    match args.output {
        OutputFormat::Text => print_text(report, discovery, args),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json_report(report, discovery, args))?
        ),
    }
    Ok(())
}

fn print_text(
    report: &SimilarityReport,
    discovery: &WorkspaceDiscovery,
    args: &SimilarityReportArgs,
) {
    let summary = &report.summary;
    println!("schema_version\t1");
    println!("threshold\t{:.6}", args.threshold);
    println!("min_node_count\t{}", args.min_node_count);
    println!("min_line_span\t{}", args.min_line_span);
    println!("comparison_scope\t{}", args.comparison_scope.label());
    println!("form_scope\t{}", args.form_scope.label());
    println!("overlap_policy\t{}", args.overlap_policy.label());
    println!("max_results\t{}", optional_usize(args.max_results));
    println!("scanned_files\t{}", discovery.files.len());
    println!("skipped_unknown\t{}", discovery.skipped_unknown_count);
    println!("skipped_hidden\t{}", discovery.skipped_hidden_count);
    println!("skipped_generated\t{}", discovery.skipped_generated_count);
    println!("skipped_symlink\t{}", discovery.skipped_symlink_count);
    println!("possible_pairs\t{}", summary.possible_pairs);
    println!("evaluated_pairs\t{}", summary.evaluated_pairs);
    println!("pruned_by_size\t{}", summary.pruned_by_size);
    println!("matched_pairs\t{}", summary.matched_pairs);
    println!("suppressed_pairs\t{}", summary.suppressed_pairs);
    println!("reported_pairs\t{}", summary.reported_pairs);
    println!("pair_count\t{}", summary.reported_pairs);
    println!("truncated\t{}", summary.truncated);

    for pair in &report.pairs {
        println!(
            "pair\tsimilarity={:.6}\tscore={:.6}",
            pair.similarity, pair.score
        );
        print_text_form("left", &pair.left);
        print_text_form("right", &pair.right);
    }
}

fn json_report(
    report: &SimilarityReport,
    discovery: &WorkspaceDiscovery,
    args: &SimilarityReportArgs,
) -> serde_json::Value {
    json!({
        "schema_version": 1,
        "pair_count": report.summary.reported_pairs,
        "options": {
            "roots": args.roots.iter().map(|root| root.display().to_string()).collect::<Vec<_>>(),
            "dialect": args.dialect.map(|dialect| Dialect::from(dialect).label()),
            "threshold": args.threshold,
            "min_node_count": args.min_node_count,
            "min_line_span": args.min_line_span,
            "comparison_scope": args.comparison_scope.label(),
            "form_scope": args.form_scope.label(),
            "overlap_policy": args.overlap_policy.label(),
            "max_results": args.max_results,
            "include_unknown": args.include_unknown,
            "include_hidden": args.include_hidden,
            "include_generated": args.include_generated,
            "max_depth": args.max_depth,
            "fail_on_duplicates": args.fail_on_duplicates,
        },
        "summary": {
            "scanned_files": discovery.files.len(),
            "skipped_unknown": discovery.skipped_unknown_count,
            "skipped_hidden": discovery.skipped_hidden_count,
            "skipped_generated": discovery.skipped_generated_count,
            "skipped_symlink": discovery.skipped_symlink_count,
            "possible_pairs": report.summary.possible_pairs,
            "evaluated_pairs": report.summary.evaluated_pairs,
            "pruned_by_size": report.summary.pruned_by_size,
            "matched_pairs": report.summary.matched_pairs,
            "suppressed_pairs": report.summary.suppressed_pairs,
            "reported_pairs": report.summary.reported_pairs,
            "truncated": report.summary.truncated,
        },
        "pairs": report.pairs.iter().map(|pair| json!({
            "similarity": pair.similarity,
            "score": pair.score,
            "left": form_json(&pair.left),
            "right": form_json(&pair.right),
        })).collect::<Vec<_>>(),
    })
}

fn optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn print_text_form(side: &str, form: &SimilarityFormReport) {
    println!(
        "\t{side}\t{}\t{}\t{}\t{}..{}\tnodes={}\thead={}",
        form.path.display(),
        form.dialect.label(),
        form.form_path,
        form.span.start().get(),
        form.span.end().get(),
        form.node_count,
        form.head.as_deref().unwrap_or("")
    );
}

fn form_json(form: &SimilarityFormReport) -> serde_json::Value {
    json!({
        "path": form.path.display().to_string(),
        "dialect": form.dialect.label(),
        "form_path": form.form_path,
        "span": { "start": form.span.start().get(), "end": form.span.end().get() },
        "node_count": form.node_count,
        "head": form.head.as_deref(),
        "text": form.text,
    })
}
