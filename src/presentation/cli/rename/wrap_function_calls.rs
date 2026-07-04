use std::fs;

use anyhow::{Context, Result};

use super::super::{detect_dialect, read_input};
use super::args::WrapFunctionCallsArgs;
use super::render::wrap::print_wrap_function_calls_report;
use super::types::{
    PendingWrapFunctionCallsFile, WrapFunctionCallsFileReport, WrapFunctionCallsPolicy,
};
use crate::application::usecase::rename::{self as rename_usecase, WrapFunctionCallsScope};

pub(in crate::presentation::cli) fn wrap_function_calls(args: WrapFunctionCallsArgs) -> Result<()> {
    if args.all_calls != args.call_paths.is_empty() {
        anyhow::bail!("wrap-function-calls requires either --all-calls or repeated --call-path");
    }

    let mut pending = Vec::with_capacity(args.files.len());
    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let scope = if args.all_calls {
            WrapFunctionCallsScope::AllCalls
        } else {
            WrapFunctionCallsScope::ExplicitPaths(args.call_paths.clone())
        };
        let plan =
            rename_usecase::plan_wrap_function_calls(rename_usecase::WrapFunctionCallsRequest {
                input: &input.text,
                dialect,
                function: args.function.clone(),
                wrapper: args.wrapper.clone(),
                wrapper_template: args.wrapper_template.clone(),
                scope,
            })
            .with_context(|| {
                format!("failed to plan wrap-function-calls for {}", file.display())
            })?;
        pending.push(PendingWrapFunctionCallsFile {
            path: file.clone(),
            dialect: plan.dialect,
            calls: plan.calls,
            skipped_already_wrapped: plan.skipped_already_wrapped,
            skipped_nested: plan.skipped_nested,
            rewritten: plan.rewritten,
            changed: plan.changed,
        });
    }

    let selected_call_count = pending.iter().map(|file| file.calls.len()).sum::<usize>();
    let policy = evaluate_wrap_function_calls_policy(selected_call_count, &args);
    if !policy.passed {
        anyhow::bail!(
            "wrap-function-calls policy failed: {}",
            policy.violations.join("; ")
        );
    }

    let mut reports = Vec::with_capacity(pending.len());
    for file in pending {
        let written = args.write && file.changed;
        if written {
            fs::write(&file.path, &file.rewritten)
                .with_context(|| format!("failed to write {}", file.path.display()))?;
        }
        reports.push(WrapFunctionCallsFileReport {
            path: file.path,
            dialect: file.dialect,
            calls: file.calls,
            skipped_already_wrapped: file.skipped_already_wrapped,
            skipped_nested: file.skipped_nested,
            changed: file.changed,
            written,
            rewritten: file.rewritten,
        });
    }

    print_wrap_function_calls_report(&reports, &args, &policy, args.output)
}

fn evaluate_wrap_function_calls_policy(
    selected_call_count: usize,
    args: &WrapFunctionCallsArgs,
) -> WrapFunctionCallsPolicy {
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
    WrapFunctionCallsPolicy {
        fail_on_no_change: args.fail_on_no_change,
        require_calls: args.require_calls,
        passed: violations.is_empty(),
        violations,
    }
}
