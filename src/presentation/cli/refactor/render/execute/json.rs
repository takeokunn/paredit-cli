use serde_json::{Value, json};

use super::super::super::super::*;
use super::super::super::types::execute::WorkspaceRefactorExecute;
use super::super::super::types::verification::RefactorVerification;

pub(super) fn print_workspace_refactor_execute_json(
    execution: &WorkspaceRefactorExecute,
) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "command": "workspace-refactor-execute",
            "mode": execution.preview.mode.label(),
            "from": execution.preview.from.as_str(),
            "to": execution.preview.to.as_str(),
            "write_requested": execution.preview.write_requested,
            "workspace": execution.preview.workspace.as_ref().map(|workspace| json!({
                "roots": workspace
                    .roots
                    .iter()
                    .map(|root| root.display().to_string())
                    .collect::<Vec<_>>(),
                "discovered_file_count": workspace.discovered_file_count,
                "skipped": {
                    "unknown": workspace.skipped_unknown_count,
                    "hidden": workspace.skipped_hidden_count,
                    "generated": workspace.skipped_generated_count,
                    "symlink": workspace.skipped_symlink_count,
                },
            })),
            "preview": {
                "summary": {
                    "file_count": execution.preview.summary.file_count,
                    "changed_file_count": execution.preview.summary.changed_file_count,
                    "unchanged_file_count": execution.preview.summary.unchanged_file_count,
                    "written_file_count": execution.preview.summary.written_file_count,
                    "definition_count": execution.preview.summary.definition_count,
                    "target_occurrence_count": execution.preview.summary.target_occurrence_count,
                    "edit_count": execution.preview.summary.edit_count,
                    "parse_error_count": execution.preview.summary.parse_error_count,
                    "all_outputs_parse": execution.preview.summary.all_outputs_parse,
                },
                "policy": {
                    "fail_on_no_change": execution.preview.policy.fail_on_no_change,
                    "fail_on_parse_error": execution.preview.policy.fail_on_parse_error,
                    "fail_on_target_conflict": execution.preview.policy.fail_on_target_conflict,
                    "require_changed_files": execution.preview.policy.require_changed_files,
                    "require_definitions": execution.preview.policy.require_definitions,
                    "require_edits": execution.preview.policy.require_edits,
                    "passed": execution.preview.policy.passed,
                    "violations": execution.preview.policy.violations.as_slice(),
                },
                "files": execution
                    .preview
                    .files
                    .iter()
                    .map(|file| json!({
                        "path": file.path.display().to_string(),
                        "dialect": file.dialect.label(),
                        "changed": file.changed,
                        "written": file.written,
                        "edit_count": file.edit_count,
                        "output_parse_ok": file.output_parse_ok,
                        "input_hash": file.input_hash.as_str(),
                        "output_hash": file.output_hash.as_str(),
                        "preview": file.preview.as_str(),
                    }))
                    .collect::<Vec<_>>(),
            },
            "pre_verification": execution
                .pre_verification
                .as_ref()
                .map(refactor_verification_json),
            "post_verification": execution
                .post_verification
                .as_ref()
                .map(refactor_verification_json),
        }))?
    );

    Ok(())
}

fn refactor_verification_json(verification: &RefactorVerification) -> Value {
    json!({
        "operation": verification.operation.label(),
        "phase": verification.phase.label(),
        "symbol": verification.symbol.as_str(),
        "new_symbol": verification.new_symbol.as_deref(),
        "passed": verification.passed,
        "before": refactor_summary_json(verification.before),
        "after": verification.after.map(refactor_summary_json),
        "checks": verification
            .checks
            .iter()
            .map(|check| json!({
                "code": check.code,
                "level": check.level.label(),
                "passed": check.passed,
                "message": check.message.as_str(),
                "count": check.count,
            }))
            .collect::<Vec<_>>(),
    })
}

fn refactor_summary_json(summary: RefactorPlanSummary) -> Value {
    json!({
        "safe_to_automate": summary.safe_to_automate,
        "file_count": summary.file_count,
        "definition_count": summary.definition_count,
        "reference_count": summary.reference_count,
        "call_count": summary.call_count,
        "inbound_edge_count": summary.inbound_edge_count,
        "outbound_edge_count": summary.outbound_edge_count,
        "non_call_reference_count": summary.non_call_reference_count,
        "signature_mismatch_count": summary.signature_mismatch_count,
    })
}
