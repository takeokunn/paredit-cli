use std::path::PathBuf;

use anyhow::Result;
use serde_json::json;

use crate::application::workspace_report::{
    WorkspaceFileMetrics, WorkspaceFileStatus, summarize_workspace_report,
};
use crate::infrastructure::workspace::WorkspaceDiscovery;
use crate::presentation::cli::OutputFormat;

use super::types::WorkspaceFileReport;

pub(super) fn print_workspace_report(
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
        OutputFormat::Text => print_text_workspace_report(roots, discovery, reports, &summary),
        OutputFormat::Json => print_json_workspace_report(roots, discovery, reports, &summary)?,
    }

    Ok(())
}

fn print_text_workspace_report(
    roots: &[PathBuf],
    discovery: &WorkspaceDiscovery,
    reports: &[WorkspaceFileReport],
    summary: &crate::application::workspace_report::WorkspaceReportSummary,
) {
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

fn print_json_workspace_report(
    roots: &[PathBuf],
    discovery: &WorkspaceDiscovery,
    reports: &[WorkspaceFileReport],
    summary: &crate::application::workspace_report::WorkspaceReportSummary,
) -> Result<()> {
    println!(
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
    );
    Ok(())
}
