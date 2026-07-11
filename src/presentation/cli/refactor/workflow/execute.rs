use super::super::super::*;
use super::super::args::*;
use super::super::render::print_workspace_refactor_execute;
use super::super::types::execute::{WorkspaceRefactorExecute, WorkspaceRefactorExecuteOutcome};
use super::preview::{
    BuildRefactorPreviewRequest, build_refactor_preview, finish_refactor_preview_failure,
    write_refactor_preview,
};
use super::verification::build_refactor_verification;
use super::workspace::discover_workspace_refactor_scope;

pub(in crate::presentation::cli) fn workspace_refactor_execute(
    args: WorkspaceRefactorExecuteArgs,
) -> Result<()> {
    let workspace = discover_workspace_refactor_scope(WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown,
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        max_depth: args.max_depth,
        exclude: Vec::new(),
    })?;
    let paths = workspace.paths;

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
        workspace: Some(workspace.workspace),
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
            None,
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

    let target_kind_hint = pre_verification
        .as_ref()
        .map(|verification| verification.target_kind);
    let post_verification = if execute_decision.run_post_verification {
        Some(build_refactor_verification(
            &paths,
            None,
            &args.from,
            Some(&args.to),
            args.operation,
            VerificationPhase::Post,
            target_kind_hint,
        )?)
    } else {
        None
    };
    let post_passed = post_verification
        .as_ref()
        .is_none_or(|verification| verification.passed);
    let outcome = WorkspaceRefactorExecuteOutcome::from_decision(
        execute_decision,
        post_verification
            .as_ref()
            .map(|verification| verification.passed),
    );
    let execution = WorkspaceRefactorExecute {
        preview,
        preflight_decision,
        execute_decision,
        outcome,
        pre_verification,
        post_verification,
    };

    print_workspace_refactor_execute(&execution, args.output)?;
    finish_refactor_preview_failure(
        "refactor workspace-execute",
        policy_passed,
        &policy_message,
        execute_decision.write_parse_refused,
    )?;

    if !pre_passed {
        anyhow::bail!("refactor workspace-execute preflight failed");
    }

    if !post_passed {
        anyhow::bail!("refactor workspace-execute post verification failed");
    }

    Ok(())
}
