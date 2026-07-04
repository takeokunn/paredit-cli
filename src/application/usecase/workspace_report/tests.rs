use crate::application::usecase::workspace_report::types::{
    WorkspaceFileMetrics, WorkspaceFileStatus,
};
use crate::application::usecase::workspace_report::workflow::summarize_workspace_report;
use crate::domain::dialect::Dialect;

#[test]
fn summary_counts_files_by_status_and_dialect() {
    let parsed = WorkspaceFileStatus::Parsed;
    let parse_error = WorkspaceFileStatus::ParseError("broken".to_owned());

    let summary = summarize_workspace_report([
        WorkspaceFileMetrics {
            dialect: Dialect::CommonLisp,
            status: &parsed,
            byte_count: 10,
            top_level_form_count: 1,
            atom_count: 3,
            definition_count: 1,
            call_count: 2,
        },
        WorkspaceFileMetrics {
            dialect: Dialect::EmacsLisp,
            status: &parse_error,
            byte_count: 5,
            top_level_form_count: 0,
            atom_count: 0,
            definition_count: 0,
            call_count: 0,
        },
    ]);

    assert_eq!(summary.file_count, 2);
    assert_eq!(summary.parsed_count, 1);
    assert_eq!(summary.parse_error_count, 1);
    assert_eq!(summary.byte_count, 15);
    assert_eq!(summary.dialect_counts["common-lisp"], 1);
    assert_eq!(summary.dialect_counts["emacs-lisp"], 1);
    assert_eq!(summary.status_counts["parsed"], 1);
    assert_eq!(summary.status_counts["parse-error"], 1);
}
