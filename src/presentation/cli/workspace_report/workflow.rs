use std::fs;

use anyhow::{Context, Result};

use crate::application::definition_report::collect_definition_forms;
use crate::application::usecase::call_report::build_call_report;
use crate::application::usecase::workspace_report::types::WorkspaceFileStatus;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::SyntaxTree;
use crate::infrastructure::workspace::{WorkspaceDiscoveryOptions, discover_workspace_files};

use super::args::WorkspaceReportArgs;
use super::render::print_workspace_report;
use super::types::WorkspaceFileReport;

pub(in crate::presentation::cli) fn workspace_report(args: WorkspaceReportArgs) -> Result<()> {
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
