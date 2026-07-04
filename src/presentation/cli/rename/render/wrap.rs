use anyhow::Result;
use serde_json::json;

use super::super::super::OutputFormat;
use super::super::args::WrapFunctionCallsArgs;
use super::super::types::{WrapFunctionCallsFileReport, WrapFunctionCallsPolicy};
use super::shared::wrap_call_sites_json;

pub(in crate::presentation::cli::rename) fn print_wrap_function_calls_report(
    reports: &[WrapFunctionCallsFileReport],
    args: &WrapFunctionCallsArgs,
    policy: &WrapFunctionCallsPolicy,
    output: OutputFormat,
) -> Result<()> {
    let call_count = reports
        .iter()
        .map(|report| report.calls.len())
        .sum::<usize>();
    let skipped_already_wrapped_count = reports
        .iter()
        .map(|report| report.skipped_already_wrapped.len())
        .sum::<usize>();
    let skipped_nested_count = reports
        .iter()
        .map(|report| report.skipped_nested.len())
        .sum::<usize>();
    match output {
        OutputFormat::Text => {
            println!("function\t{}", args.function);
            println!("wrapper\t{}", args.wrapper);
            if let Some(template) = &args.wrapper_template {
                println!("wrapperTemplate\t{template}");
            }
            println!("callCount\t{call_count}");
            println!("skippedAlreadyWrappedCount\t{skipped_already_wrapped_count}");
            println!("skippedNestedCount\t{skipped_nested_count}");
            println!("passed\t{}", policy.passed);
            for report in reports {
                println!(
                    "{}\t{}\tcalls={}\tchanged={}\twritten={}",
                    report.path.display(),
                    report.dialect.label(),
                    report.calls.len(),
                    report.changed,
                    report.written
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "function": args.function.as_str(),
                "wrapper": args.wrapper.as_str(),
                "wrapperTemplate": args.wrapper_template.as_deref(),
                "allCalls": args.all_calls,
                "callPaths": args.call_paths.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "write": args.write,
                "callCount": call_count,
                "skippedAlreadyWrappedCount": skipped_already_wrapped_count,
                "skippedNestedCount": skipped_nested_count,
                "policy": {
                    "failOnNoChange": policy.fail_on_no_change,
                    "requireCalls": policy.require_calls,
                    "passed": policy.passed,
                    "violations": policy.violations,
                },
                "files": reports.iter().map(|report| json!({
                    "path": report.path.display().to_string(),
                    "dialect": report.dialect.label(),
                    "callCount": report.calls.len(),
                    "skippedAlreadyWrappedCount": report.skipped_already_wrapped.len(),
                    "skippedNestedCount": report.skipped_nested.len(),
                    "changed": report.changed,
                    "written": report.written,
                    "calls": wrap_call_sites_json(&report.calls),
                    "skippedAlreadyWrapped": wrap_call_sites_json(&report.skipped_already_wrapped),
                    "skippedNested": wrap_call_sites_json(&report.skipped_nested),
                    "rewritten": report.rewritten.as_str(),
                })).collect::<Vec<_>>(),
            }))?
        ),
    }
    Ok(())
}
