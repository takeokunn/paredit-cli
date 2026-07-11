use anyhow::{Context, Result};

use super::super::{detect_dialect, read_input, write_files_with_rollback};
use super::args::ReplaceFunctionCallsArgs;
use super::render::replace_call::print_replace_function_calls_report;
use super::types::{
    PendingReplaceFunctionCallsFile, ReplaceFunctionCallsFileReport, ReplaceFunctionCallsPolicy,
};
use crate::application::usecase::rename::{self as rename_usecase, ReplaceFunctionCallsScope};

pub(in crate::presentation::cli) fn replace_function_calls(
    args: ReplaceFunctionCallsArgs,
) -> Result<()> {
    if args.all_calls != args.call_paths.is_empty() {
        anyhow::bail!("replace-function-calls requires either --all-calls or repeated --call-path");
    }

    let mut pending = Vec::with_capacity(args.files.len());
    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let scope = if args.all_calls {
            ReplaceFunctionCallsScope::AllCalls
        } else {
            ReplaceFunctionCallsScope::ExplicitPaths(args.call_paths.clone())
        };
        let plan = rename_usecase::plan_replace_function_calls(
            rename_usecase::ReplaceFunctionCallsRequest {
                input: &input.text,
                dialect,
                from: args.from.clone(),
                to: args.to.clone(),
                scope,
            },
        )
        .with_context(|| {
            format!(
                "failed to plan replace-function-calls for {}",
                file.display()
            )
        })?;
        pending.push(PendingReplaceFunctionCallsFile {
            path: file.clone(),
            dialect: plan.dialect,
            calls: plan.calls,
            rewritten: plan.rewritten,
            changed: plan.changed,
        });
    }

    let selected_call_count = pending.iter().map(|file| file.calls.len()).sum::<usize>();
    let policy = evaluate_replace_function_calls_policy(selected_call_count, &args);
    if !policy.passed {
        anyhow::bail!(
            "replace-function-calls policy failed: {}",
            policy.violations.join("; ")
        );
    }

    let written_files = pending
        .iter()
        .filter(|file| args.write && file.changed)
        .map(|file| (file.path.clone(), file.rewritten.clone()))
        .collect::<Vec<_>>();
    if !written_files.is_empty() {
        write_files_with_rollback(written_files)?;
    }

    let mut reports = Vec::with_capacity(pending.len());
    for file in pending {
        let written = args.write && file.changed;
        reports.push(ReplaceFunctionCallsFileReport {
            path: file.path,
            dialect: file.dialect,
            calls: file.calls,
            changed: file.changed,
            written,
            rewritten: file.rewritten,
        });
    }

    print_replace_function_calls_report(&reports, &args, &policy, args.output)
}

fn evaluate_replace_function_calls_policy(
    selected_call_count: usize,
    args: &ReplaceFunctionCallsArgs,
) -> ReplaceFunctionCallsPolicy {
    let mut violations = Vec::new();
    if args.fail_on_no_change && selected_call_count == 0 {
        violations.push("no selected call site changed".to_owned());
    }
    if let Some(required) = args.require_calls {
        if selected_call_count < required {
            violations.push(format!(
                "expected at least {required} changed call sites but found {selected_call_count}"
            ));
        }
    }
    ReplaceFunctionCallsPolicy {
        fail_on_no_change: args.fail_on_no_change,
        require_calls: args.require_calls,
        passed: violations.is_empty(),
        violations,
    }
}
