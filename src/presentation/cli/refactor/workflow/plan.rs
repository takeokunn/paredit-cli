use super::super::super::*;
use super::super::args::*;
use super::super::render::print_refactor_plan;
use super::super::types::plan::{
    RefactorPlan, RefactorPlanPolicyOptions, WorkspaceRefactorPlanDiscovery,
};

pub(in crate::presentation::cli) fn refactor_plan(args: RefactorPlanArgs) -> Result<()> {
    emit_refactor_plan(RefactorPlanEmission {
        paths: &args.files,
        dialect: args.dialect,
        symbol: &args.symbol,
        operation: args.operation,
        policy_options: RefactorPlanPolicyOptions {
            fail_on_blocking_gate: args.fail_on_blocking_gate,
            require_definitions: args.require_definitions,
            require_references: args.require_references,
        },
        workspace: None,
        output: args.output,
        failure_label: "refactor-plan",
    })
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

    emit_refactor_plan(RefactorPlanEmission {
        paths: &discovery.files,
        dialect: None,
        symbol: &args.symbol,
        operation: args.operation,
        policy_options: RefactorPlanPolicyOptions {
            fail_on_blocking_gate: args.fail_on_blocking_gate,
            require_definitions: args.require_definitions,
            require_references: args.require_references,
        },
        workspace: Some(workspace),
        output: args.output,
        failure_label: "workspace-refactor-plan",
    })
}

struct RefactorPlanEmission<'a> {
    paths: &'a [PathBuf],
    dialect: Option<DialectArg>,
    symbol: &'a SymbolName,
    operation: RefactorOperation,
    policy_options: RefactorPlanPolicyOptions,
    workspace: Option<WorkspaceRefactorPlanDiscovery>,
    output: OutputFormat,
    failure_label: &'static str,
}

fn emit_refactor_plan(request: RefactorPlanEmission<'_>) -> Result<()> {
    let RefactorPlanEmission {
        paths,
        dialect,
        symbol,
        operation,
        policy_options,
        workspace,
        output,
        failure_label,
    } = request;
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
    let automation = decision.automation;
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
        automation,
    };

    print_refactor_plan(&plan, output)?;

    if !policy_passed {
        anyhow::bail!("{failure_label} policy failed: {policy_message}");
    }

    Ok(())
}
