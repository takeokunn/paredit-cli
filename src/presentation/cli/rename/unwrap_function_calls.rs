use anyhow::{Context, Result};

use super::super::{detect_dialect, read_input, write_files_with_rollback};
use super::args::UnwrapFunctionCallsArgs;
use super::render::unwrap::print_unwrap_function_calls_report;
use super::types::{
    PendingUnwrapFunctionCallsFile, UnwrapFunctionCallsFileReport, UnwrapFunctionCallsPolicy,
};
use crate::application::usecase::rename::{self as rename_usecase, UnwrapFunctionCallsScope};

pub(in crate::presentation::cli) fn unwrap_function_calls(
    args: UnwrapFunctionCallsArgs,
) -> Result<()> {
    if args.all_calls != args.call_paths.is_empty() {
        anyhow::bail!("unwrap-function-calls requires either --all-calls or repeated --call-path");
    }

    let mut pending = Vec::with_capacity(args.files.len());
    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let scope = if args.all_calls {
            UnwrapFunctionCallsScope::AllCalls
        } else {
            UnwrapFunctionCallsScope::ExplicitPaths(args.call_paths.clone())
        };
        let plan = rename_usecase::plan_unwrap_function_calls(
            rename_usecase::UnwrapFunctionCallsRequest {
                input: &input.text,
                dialect,
                function: args.function.clone(),
                wrapper: args.wrapper.clone(),
                scope,
            },
        )
        .with_context(|| {
            format!(
                "failed to plan unwrap-function-calls for {}",
                file.display()
            )
        })?;
        pending.push(PendingUnwrapFunctionCallsFile {
            path: file.clone(),
            dialect: plan.dialect,
            calls: plan.calls,
            skipped_non_unary_wrapper: plan.skipped_non_unary_wrapper,
            skipped_nested: plan.skipped_nested,
            rewritten: plan.rewritten,
            changed: plan.changed,
        });
    }

    let selected_call_count = pending.iter().map(|file| file.calls.len()).sum::<usize>();
    let policy = evaluate_unwrap_function_calls_policy(selected_call_count, &args);
    if !policy.passed {
        anyhow::bail!(
            "unwrap-function-calls policy failed: {}",
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
        reports.push(UnwrapFunctionCallsFileReport {
            path: file.path,
            dialect: file.dialect,
            calls: file.calls,
            skipped_non_unary_wrapper: file.skipped_non_unary_wrapper,
            skipped_nested: file.skipped_nested,
            changed: file.changed,
            written,
            rewritten: file.rewritten,
        });
    }

    print_unwrap_function_calls_report(&reports, &args, &policy, args.output)
}

fn evaluate_unwrap_function_calls_policy(
    selected_call_count: usize,
    args: &UnwrapFunctionCallsArgs,
) -> UnwrapFunctionCallsPolicy {
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
    UnwrapFunctionCallsPolicy {
        fail_on_no_change: args.fail_on_no_change,
        require_calls: args.require_calls,
        passed: violations.is_empty(),
        violations,
    }
}
