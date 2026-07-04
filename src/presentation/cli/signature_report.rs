use super::*;

use crate::application::signature_report::{
    SignatureCallStatus, SignatureReportFile, SignatureReportPolicy, SignatureReportSource,
    build_signature_reports, evaluate_signature_report_policy,
};

#[derive(Debug, Args)]
pub(super) struct SignatureReportArgs {
    /// Files to scan.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Exact callable symbol to report. Reports every non-definition call when omitted.
    #[arg(long)]
    symbol: Option<SymbolName>,
    /// Exit with failure when any discovered call has too few or too many arguments.
    #[arg(long)]
    fail_on_mismatch: bool,
    /// Require at least this many matching callable definitions.
    #[arg(long)]
    require_definitions: Option<usize>,
    /// Require at least this many discovered call sites.
    #[arg(long)]
    require_calls: Option<usize>,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

pub(super) fn signature_report(args: SignatureReportArgs) -> Result<()> {
    let symbol = args.symbol.as_ref();
    let mut sources = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        sources.push(SignatureReportSource {
            path: file.clone(),
            dialect,
            tree,
        });
    }

    let reports = build_signature_reports(sources, symbol)?;
    let policy = evaluate_signature_report_policy(
        &reports,
        args.fail_on_mismatch,
        args.require_definitions,
        args.require_calls,
    );
    print_signature_report(&reports, symbol, &policy, args.output)?;
    if !policy.passed {
        anyhow::bail!("signature-report policy failed");
    }
    Ok(())
}

fn print_signature_report(
    reports: &[SignatureReportFile],
    symbol: Option<&SymbolName>,
    policy: &SignatureReportPolicy,
    output: OutputFormat,
) -> Result<()> {
    let mut by_status = BTreeMap::<SignatureCallStatus, usize>::new();
    for item in reports.iter().flat_map(|report| &report.calls) {
        *by_status.entry(item.status).or_default() += 1;
    }

    match output {
        OutputFormat::Text => {
            println!("symbol\t{}", symbol.map_or("<all>", SymbolName::as_str));
            println!("files\t{}", reports.len());
            println!("definition_count\t{}", policy.definition_count);
            println!("call_count\t{}", policy.call_count);
            println!("mismatch_count\t{}", policy.mismatch_count);
            println!("policy_passed\t{}", policy.passed);
            for violation in &policy.violations {
                println!("policy_violation\t{violation}");
            }
            for (status, count) in &by_status {
                println!("status\t{}\t{count}", status.label());
            }
            for report in reports {
                println!(
                    "{}\t{}\tdefinitions={}\tcalls={}",
                    report.path.display(),
                    report.dialect.label(),
                    report.definitions.len(),
                    report.calls.len()
                );
                for definition in &report.definitions {
                    println!(
                        "\tdefinition\t{}\t{}\t{}..{}\tparams={}",
                        definition.path,
                        definition.name.as_deref().unwrap_or(""),
                        definition.span.start().get(),
                        definition.span.end().get(),
                        definition
                            .parameter_count
                            .map(|count| count.to_string())
                            .unwrap_or_default(),
                    );
                }
                for item in &report.calls {
                    let expected = item
                        .expected_parameter_count
                        .map(|count| count.to_string())
                        .unwrap_or_default();
                    let enclosing = item
                        .call
                        .enclosing_definition
                        .as_deref()
                        .unwrap_or("<none>");
                    println!(
                        "\tcall\t{}\t{}\t{}..{}\targs={}\texpected={}\tstatus={}\tenclosing={}",
                        item.call.path,
                        item.call.head,
                        item.call.span.start().get(),
                        item.call.span.end().get(),
                        item.call.argument_count,
                        expected,
                        item.status.label(),
                        enclosing,
                    );
                }
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "symbol": symbol.map(SymbolName::as_str),
                "file_count": reports.len(),
                "definition_count": policy.definition_count,
                "call_count": policy.call_count,
                "mismatch_count": policy.mismatch_count,
                "policy": {
                    "fail_on_mismatch": policy.fail_on_mismatch,
                    "require_definitions": policy.require_definitions,
                    "require_calls": policy.require_calls,
                    "passed": policy.passed,
                    "violations": policy.violations,
                },
                "by_status": by_status
                    .iter()
                    .map(|(status, count)| json!({
                        "status": status.label(),
                        "count": count,
                    }))
                    .collect::<Vec<_>>(),
                "files": reports
                    .iter()
                    .map(|report| json!({
                        "path": report.path.display().to_string(),
                        "dialect": report.dialect.label(),
                        "definition_count": report.definitions.len(),
                        "call_count": report.calls.len(),
                        "definitions": report
                            .definitions
                            .iter()
                            .map(|definition| json!({
                                "path": definition.path.as_str(),
                                "span": {
                                    "start": definition.span.start().get(),
                                    "end": definition.span.end().get(),
                                },
                                "head": definition.head.as_str(),
                                "name": definition.name.as_deref(),
                                "category": definition.category.label(),
                                "parameterCount": definition.parameter_count,
                            }))
                            .collect::<Vec<_>>(),
                        "calls": report
                            .calls
                            .iter()
                            .map(|item| json!({
                                "path": item.call.path.as_str(),
                                "span": {
                                    "start": item.call.span.start().get(),
                                    "end": item.call.span.end().get(),
                                },
                                "head": item.call.head.as_str(),
                                "argumentCount": item.call.argument_count,
                                "expectedParameterCount": item.expected_parameter_count,
                                "status": item.status.label(),
                                "enclosingDefinition": item.call.enclosing_definition.as_deref(),
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }

    Ok(())
}
