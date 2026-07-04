mod build;
mod failure;
mod write;

use super::super::super::*;
use super::super::args::*;
use super::super::render::print_refactor_preview;
use super::super::types::plan::WorkspaceRefactorPlanDiscovery;

pub(in crate::presentation::cli::refactor::workflow) use build::{
    build_refactor_preview, BuildRefactorPreviewRequest,
};
pub(in crate::presentation::cli::refactor::workflow) use failure::finish_refactor_preview_failure;
pub(in crate::presentation::cli::refactor::workflow) use write::write_refactor_preview;

pub(in crate::presentation::cli) fn refactor_preview(args: RefactorPreviewArgs) -> Result<()> {
    emit_refactor_preview(RefactorPreviewEmission {
        paths: &args.files,
        dialect: args.dialect,
        from: &args.from,
        to: &args.to,
        mode: args.mode,
        max_preview_bytes: args.max_preview_bytes,
        write: args.write,
        policy_options: RefactorPreviewPolicyOptions {
            fail_on_no_change: args.fail_on_no_change,
            fail_on_parse_error: args.fail_on_parse_error,
            fail_on_target_conflict: args.fail_on_target_conflict,
            require_changed_files: args.require_changed_files,
            require_definitions: args.require_definitions,
            require_edits: args.require_edits,
        },
        workspace: None,
        output: args.output,
        failure_label: "refactor-preview",
    })
}

pub(in crate::presentation::cli) fn workspace_refactor_preview(
    args: WorkspaceRefactorPreviewArgs,
) -> Result<()> {
    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown,
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        max_depth: args.max_depth,
    })?;
    let workspace = WorkspaceRefactorPlanDiscovery {
        roots: args.roots,
        discovered_file_count: discovery.files.len(),
        skipped_unknown_count: discovery.skipped_unknown_count,
        skipped_hidden_count: discovery.skipped_hidden_count,
        skipped_generated_count: discovery.skipped_generated_count,
        skipped_symlink_count: discovery.skipped_symlink_count,
    };

    emit_refactor_preview(RefactorPreviewEmission {
        paths: &discovery.files,
        dialect: None,
        from: &args.from,
        to: &args.to,
        mode: args.mode,
        max_preview_bytes: args.max_preview_bytes,
        write: args.write,
        policy_options: RefactorPreviewPolicyOptions {
            fail_on_no_change: args.fail_on_no_change,
            fail_on_parse_error: args.fail_on_parse_error,
            fail_on_target_conflict: args.fail_on_target_conflict,
            require_changed_files: args.require_changed_files,
            require_definitions: args.require_definitions,
            require_edits: args.require_edits,
        },
        workspace: Some(workspace),
        output: args.output,
        failure_label: "workspace-refactor-preview",
    })
}

struct RefactorPreviewEmission<'a> {
    paths: &'a [PathBuf],
    dialect: Option<DialectArg>,
    from: &'a SymbolName,
    to: &'a SymbolName,
    mode: RefactorPreviewMode,
    max_preview_bytes: usize,
    write: bool,
    policy_options: RefactorPreviewPolicyOptions,
    workspace: Option<WorkspaceRefactorPlanDiscovery>,
    output: OutputFormat,
    failure_label: &'static str,
}

fn emit_refactor_preview(request: RefactorPreviewEmission<'_>) -> Result<()> {
    let RefactorPreviewEmission {
        paths,
        dialect,
        from,
        to,
        mode,
        max_preview_bytes,
        write,
        policy_options,
        workspace,
        output,
        failure_label,
    } = request;
    let mut preview = build_refactor_preview(BuildRefactorPreviewRequest {
        paths,
        dialect,
        from,
        to,
        mode,
        max_preview_bytes,
        write,
        policy_options,
        workspace,
    })?;
    let policy_passed = preview.policy.passed;
    let policy_message = preview.policy.violations.join("; ");
    let write_parse_refused = write && !preview.summary.all_outputs_parse;

    if policy_passed && !write_parse_refused {
        write_refactor_preview(&mut preview)?;
    }

    print_refactor_preview(&preview, output)?;
    finish_refactor_preview_failure(
        failure_label,
        policy_passed,
        &policy_message,
        write_parse_refused,
    )
}
