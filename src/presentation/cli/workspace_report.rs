use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use serde_json::json;

use crate::application::call_report::build_call_report;
use crate::application::definition_report::collect_definition_forms;
use crate::application::workspace_report::{
    WorkspaceFileMetrics, WorkspaceFileStatus, summarize_workspace_report,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::SyntaxTree;
use crate::infrastructure::workspace::{
    WorkspaceDiscovery, WorkspaceDiscoveryOptions, discover_workspace_files,
};

use super::OutputFormat;

#[derive(Debug, Args)]
pub(super) struct WorkspaceReportArgs {
    /// Files or directories to scan recursively.
    #[arg(required = true)]
    roots: Vec<PathBuf>,
    /// Include files whose extension does not identify a known Lisp dialect.
    #[arg(long)]
    include_unknown: bool,
    /// Include hidden directories and files.
    #[arg(long)]
    include_hidden: bool,
    /// Include generated or dependency directories such as target and node_modules.
    #[arg(long)]
    include_generated: bool,
    /// Maximum directory recursion depth from each root directory.
    #[arg(long)]
    max_depth: Option<usize>,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

#[derive(Debug)]
struct WorkspaceFileReport {
    path: PathBuf,
    dialect: Dialect,
    status: WorkspaceFileStatus,
    byte_count: usize,
    top_level_form_count: usize,
    atom_count: usize,
    definition_count: usize,
    call_count: usize,
    package: Option<String>,
}

pub(super) fn workspace_report(args: WorkspaceReportArgs) -> Result<()> {
    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown,
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        max_depth: args.max_depth,
    })?;
    let mut reports = Vec::with_capacity(discovery.files.len());

    for file in &discovery.files {
        let text = fs::read_to_string(file)
            .with_context(|| format!("failed to read {}", file.display()))?;
        let dialect = Dialect::detect(Some(file.as_path()), None);
        let byte_count = text.len();

        match SyntaxTree::parse(&text) {
            Ok(tree) => {
                let (package, definitions) = collect_definition_forms(&tree, dialect)
                    .with_context(|| format!("failed to analyze {}", file.display()))?;
                let calls = build_call_report(&tree, dialect, None, false)
                    .with_context(|| format!("failed to collect calls in {}", file.display()))?;

                reports.push(WorkspaceFileReport {
                    path: file.clone(),
                    dialect,
                    status: WorkspaceFileStatus::Parsed,
                    byte_count,
                    top_level_form_count: tree.root_children().len(),
                    atom_count: tree.atom_occurrences().len(),
                    definition_count: definitions.len(),
                    call_count: calls.len(),
                    package,
                });
            }
            Err(error) => {
                reports.push(WorkspaceFileReport {
                    path: file.clone(),
                    dialect,
                    status: WorkspaceFileStatus::ParseError(error.to_string()),
                    byte_count,
                    top_level_form_count: 0,
                    atom_count: 0,
                    definition_count: 0,
                    call_count: 0,
                    package: None,
                });
            }
        }
    }

    print_workspace_report(&args.roots, &discovery, &reports, args.output)
}

fn print_workspace_report(
    roots: &[PathBuf],
    discovery: &WorkspaceDiscovery,
    reports: &[WorkspaceFileReport],
    output: OutputFormat,
) -> Result<()> {
    let summary = summarize_workspace_report(reports.iter().map(|report| WorkspaceFileMetrics {
        dialect: report.dialect,
        status: &report.status,
        byte_count: report.byte_count,
        top_level_form_count: report.top_level_form_count,
        atom_count: report.atom_count,
        definition_count: report.definition_count,
        call_count: report.call_count,
    }));

    match output {
        OutputFormat::Text => {
            println!(
                "roots\t{}",
                roots
                    .iter()
                    .map(|root| root.display().to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            println!("files\t{}", summary.file_count);
            println!("parsed\t{}", summary.parsed_count);
            println!("parse_errors\t{}", summary.parse_error_count);
            println!("bytes\t{}", summary.byte_count);
            println!("top_level_forms\t{}", summary.top_level_form_count);
            println!("atoms\t{}", summary.atom_count);
            println!("definitions\t{}", summary.definition_count);
            println!("calls\t{}", summary.call_count);
            println!("skipped_unknown\t{}", discovery.skipped_unknown_count);
            println!("skipped_hidden\t{}", discovery.skipped_hidden_count);
            println!("skipped_generated\t{}", discovery.skipped_generated_count);
            println!("skipped_symlink\t{}", discovery.skipped_symlink_count);
            for (dialect, count) in &summary.dialect_counts {
                println!("dialect\t{dialect}\t{count}");
            }
            for (status, count) in &summary.status_counts {
                println!("status\t{status}\t{count}");
            }
            for report in reports {
                println!(
                    "{}\t{}\t{}\tdefinitions={}\tcalls={}",
                    report.path.display(),
                    report.dialect.label(),
                    report.status.label(),
                    report.definition_count,
                    report.call_count
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "roots": roots
                    .iter()
                    .map(|root| root.display().to_string())
                    .collect::<Vec<_>>(),
                "file_count": summary.file_count,
                "parsed_count": summary.parsed_count,
                "parse_error_count": summary.parse_error_count,
                "byte_count": summary.byte_count,
                "top_level_form_count": summary.top_level_form_count,
                "atom_count": summary.atom_count,
                "definition_count": summary.definition_count,
                "call_count": summary.call_count,
                "dialects": summary.dialect_counts
                    .iter()
                    .map(|(dialect, count)| json!({
                        "dialect": dialect,
                        "count": count,
                    }))
                    .collect::<Vec<_>>(),
                "statuses": summary.status_counts
                    .iter()
                    .map(|(status, count)| json!({
                        "status": status,
                        "count": count,
                    }))
                    .collect::<Vec<_>>(),
                "skipped": {
                    "unknown": discovery.skipped_unknown_count,
                    "hidden": discovery.skipped_hidden_count,
                    "generated": discovery.skipped_generated_count,
                    "symlink": discovery.skipped_symlink_count,
                },
                "files": reports
                    .iter()
                    .map(|report| json!({
                        "path": report.path.display().to_string(),
                        "dialect": report.dialect.label(),
                        "status": report.status.label(),
                        "error": match &report.status {
                            WorkspaceFileStatus::Parsed => None,
                            WorkspaceFileStatus::ParseError(error) => Some(error.as_str()),
                        },
                        "byte_count": report.byte_count,
                        "top_level_form_count": report.top_level_form_count,
                        "atom_count": report.atom_count,
                        "definition_count": report.definition_count,
                        "call_count": report.call_count,
                        "package": report.package.as_deref(),
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }

    Ok(())
}
