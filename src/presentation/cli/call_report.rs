use super::*;

use crate::application::call_report::{CallReportItem, build_call_report};

#[derive(Debug, Args)]
pub(super) struct CallReportArgs {
    /// Files to scan.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Exact list-head symbol to report. Reports every non-definition call when omitted.
    #[arg(long)]
    symbol: Option<SymbolName>,
    /// Include definition-like forms such as defun and defmacro in the report.
    #[arg(long)]
    include_definitions: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

#[derive(Debug)]
struct CallReportFile {
    path: PathBuf,
    dialect: Dialect,
    calls: Vec<CallReportItem>,
}

pub(super) fn call_report(args: CallReportArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        let calls = build_call_report(
            &tree,
            dialect,
            args.symbol.as_ref(),
            args.include_definitions,
        )?;

        reports.push(CallReportFile {
            path: file.clone(),
            dialect,
            calls,
        });
    }

    print_call_report(
        &reports,
        args.symbol.as_ref(),
        args.include_definitions,
        args.output,
    )
}

fn print_call_report(
    reports: &[CallReportFile],
    symbol: Option<&SymbolName>,
    include_definitions: bool,
    output: OutputFormat,
) -> Result<()> {
    let total_count = reports
        .iter()
        .map(|report| report.calls.len())
        .sum::<usize>();

    match output {
        OutputFormat::Text => {
            println!("symbol\t{}", symbol.map_or("<all>", SymbolName::as_str));
            println!("include_definitions\t{include_definitions}");
            println!("files\t{}", reports.len());
            println!("total_count\t{total_count}");
            for report in reports {
                println!(
                    "{}\t{}\tcount={}",
                    report.path.display(),
                    report.dialect.label(),
                    report.calls.len()
                );
                for call in &report.calls {
                    let category = call
                        .category
                        .map(DefinitionCategory::label)
                        .unwrap_or("call");
                    let enclosing = call.enclosing_definition.as_deref().unwrap_or("<none>");
                    println!(
                        "\t{}\t{}\t{}..{}\targs={}\tcategory={}\tenclosing={}",
                        call.path,
                        call.head,
                        call.span.start().get(),
                        call.span.end().get(),
                        call.argument_count,
                        category,
                        enclosing
                    );
                }
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "symbol": symbol.map(SymbolName::as_str),
                "includeDefinitions": include_definitions,
                "file_count": reports.len(),
                "total_count": total_count,
                "files": reports
                    .iter()
                    .map(|report| json!({
                        "path": report.path.display().to_string(),
                        "dialect": report.dialect.label(),
                        "count": report.calls.len(),
                        "calls": report
                            .calls
                            .iter()
                            .map(|call| json!({
                                "path": call.path.as_str(),
                                "span": {
                                    "start": call.span.start().get(),
                                    "end": call.span.end().get(),
                                },
                                "head": call.head.as_str(),
                                "argumentCount": call.argument_count,
                                "category": call.category.map(DefinitionCategory::label),
                                "enclosingDefinition": call.enclosing_definition.as_deref(),
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }

    Ok(())
}
