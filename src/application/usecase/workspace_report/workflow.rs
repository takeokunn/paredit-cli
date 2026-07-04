use std::collections::BTreeMap;

use crate::application::usecase::workspace_report::types::{
    WorkspaceFileMetrics, WorkspaceReportSummary,
};

pub fn summarize_workspace_report<'a>(
    files: impl IntoIterator<Item = WorkspaceFileMetrics<'a>>,
) -> WorkspaceReportSummary {
    let mut summary = WorkspaceReportSummary {
        file_count: 0,
        parsed_count: 0,
        parse_error_count: 0,
        byte_count: 0,
        top_level_form_count: 0,
        atom_count: 0,
        definition_count: 0,
        call_count: 0,
        dialect_counts: BTreeMap::new(),
        status_counts: BTreeMap::new(),
    };

    for file in files {
        summary.file_count += 1;
        if file.status.is_parsed() {
            summary.parsed_count += 1;
        } else {
            summary.parse_error_count += 1;
        }
        summary.byte_count += file.byte_count;
        summary.top_level_form_count += file.top_level_form_count;
        summary.atom_count += file.atom_count;
        summary.definition_count += file.definition_count;
        summary.call_count += file.call_count;
        *summary
            .dialect_counts
            .entry(file.dialect.label())
            .or_default() += 1;
        *summary
            .status_counts
            .entry(file.status.label())
            .or_default() += 1;
    }

    summary
}
