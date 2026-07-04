use super::super::super::*;
use super::super::args::*;
use super::super::render::print_refactor_plan;
use super::super::types::plan::{
    RefactorPlan, RefactorPlanPolicyOptions, WorkspaceRefactorPlanDiscovery,
};

pub(in crate::presentation::cli) fn refactor_plan(args: RefactorPlanArgs) -> Result<()> {
    emit_refactor_plan(
        &args.files,
        args.dialect,
        &args.symbol,
        args.operation,
        RefactorPlanPolicyOptions {
            fail_on_blocking_gate: args.fail_on_blocking_gate,
            require_definitions: args.require_definitions,
            require_references: args.require_references,
        },
        None,
        args.output,
        "refactor-plan",
    )
}

pub(in crate::presentation::cli) fn workspace_refactor_plan(
    args: WorkspaceRefactorPlanArgs,
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

    emit_refactor_plan(
        &discovery.files,
        None,
        &args.symbol,
        args.operation,
        RefactorPlanPolicyOptions {
            fail_on_blocking_gate: args.fail_on_blocking_gate,
            require_definitions: args.require_definitions,
            require_references: args.require_references,
        },
        Some(workspace),
        args.output,
        "workspace-refactor-plan",
    )
}

fn emit_refactor_plan(
    paths: &[PathBuf],
    dialect: Option<DialectArg>,
    symbol: &SymbolName,
    operation: RefactorOperation,
    policy_options: RefactorPlanPolicyOptions,
    workspace: Option<WorkspaceRefactorPlanDiscovery>,
    output: OutputFormat,
    failure_label: &'static str,
) -> Result<()> {
    let files = impact_report::workflow::collect_impact_reports(paths, dialect, symbol)?;
    let summary = summarize_impact_reports(&files);
    let operation = ApplicationRefactorOperation::from(operation);
    let decision = build_refactor_plan_decision(RefactorPlanRequest {
        operation,
        symbol: symbol.as_str(),
        files: paths,
        summary,
        policy: RefactorPlanPolicyRequest {
            fail_on_blocking_gate: policy_options.fail_on_blocking_gate,
            require_definitions: policy_options.require_definitions,
            require_references: policy_options.require_references,
        },
        risks: raw_refactor_risks(&summary),
    });
    let policy = decision.policy;
    let policy_passed = policy.passed;
    let policy_message = policy.violations.join("; ");
    let plan = RefactorPlan {
        operation,
        symbol: symbol.as_str().to_owned(),
        workspace,
        files,
        gates: decision.gates,
        steps: decision.steps,
        policy,
    };

    print_refactor_plan(&plan, output)?;

    if !policy_passed {
        anyhow::bail!("{failure_label} policy failed: {policy_message}");
    }

    Ok(())
}
