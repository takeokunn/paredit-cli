use super::super::super::*;
use super::super::args::*;
use super::super::render::print_workspace_refactor_execute;
use super::super::types::execute::WorkspaceRefactorExecute;
use super::super::types::plan::WorkspaceRefactorPlanDiscovery;
use super::preview::{
    build_refactor_preview, finish_refactor_preview_failure, write_refactor_preview,
    BuildRefactorPreviewRequest,
};
use super::verification::build_refactor_verification;

pub(in crate::presentation::cli) fn workspace_refactor_execute(
    args: WorkspaceRefactorExecuteArgs,
) -> Result<()> {
    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown,
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        max_depth: args.max_depth,
    })?;
    let paths = discovery.files;
    let workspace = WorkspaceRefactorPlanDiscovery {
        roots: args.roots,
        discovered_file_count: paths.len(),
        skipped_unknown_count: discovery.skipped_unknown_count,
        skipped_hidden_count: discovery.skipped_hidden_count,
        skipped_generated_count: discovery.skipped_generated_count,
        skipped_symlink_count: discovery.skipped_symlink_count,
    };

    let mut preview = build_refactor_preview(BuildRefactorPreviewRequest {
        paths: &paths,
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
    })?;

    let policy_passed = preview.policy.passed;
    let policy_message = preview.policy.violations.join("; ");
    let preflight_decision = build_refactor_execute_decision(RefactorExecuteGateInputs {
        write_requested: args.write,
        policy_passed,
        outputs_parse: preview.summary.all_outputs_parse,
        preflight_passed: true,
    });

    let pre_verification = if preflight_decision.run_pre_verification {
        Some(build_refactor_verification(
            &paths,
            None,
            &args.from,
            Some(&args.to),
            args.operation,
            VerificationPhase::Pre,
        )?)
    } else {
        None
    };
    let pre_passed = pre_verification
        .as_ref()
        .is_none_or(|verification| verification.passed);
    let execute_decision = build_refactor_execute_decision(RefactorExecuteGateInputs {
        write_requested: args.write,
        policy_passed,
        outputs_parse: preview.summary.all_outputs_parse,
        preflight_passed: pre_passed,
    });

    if execute_decision.apply_preview {
        write_refactor_preview(&mut preview)?;
    }

    let post_verification = if execute_decision.run_post_verification {
        Some(build_refactor_verification(
            &paths,
            None,
            &args.from,
            Some(&args.to),
            args.operation,
            VerificationPhase::Post,
        )?)
    } else {
        None
    };
    let post_passed = post_verification
        .as_ref()
        .is_none_or(|verification| verification.passed);
    let execution = WorkspaceRefactorExecute {
        preview,
        preflight_decision,
        execute_decision,
        pre_verification,
        post_verification,
    };

    print_workspace_refactor_execute(&execution, args.output)?;
    finish_refactor_preview_failure(
        "workspace-refactor-execute",
        policy_passed,
        &policy_message,
        execute_decision.write_parse_refused,
    )?;

    if !pre_passed {
        anyhow::bail!("workspace-refactor-execute preflight failed");
    }

    if !post_passed {
        anyhow::bail!("workspace-refactor-execute post verification failed");
    }

    Ok(())
}
