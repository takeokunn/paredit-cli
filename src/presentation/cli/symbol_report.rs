use super::*;

#[derive(Debug, Args)]
pub(super) struct SymbolQueryArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Exact symbol atom to find.
    #[arg(long)]
    symbol: SymbolName,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

#[derive(Debug, Args)]
pub(super) struct SymbolReportArgs {
    /// Files to scan.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Exact symbol atom to report.
    #[arg(long)]
    symbol: SymbolName,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

#[derive(Debug)]
struct SymbolReportFile {
    path: PathBuf,
    dialect: Dialect,
    occurrences: Vec<SymbolReportOccurrence>,
}

#[derive(Debug)]
struct SymbolReportOccurrence {
    path: String,
    span: ByteSpan,
    context: Option<SymbolOccurrenceContext>,
}

#[derive(Debug)]
struct SymbolOccurrenceContext {
    path: String,
    span: ByteSpan,
    head: Option<String>,
    definition_like: bool,
}

pub(super) fn find_symbol(args: SymbolQueryArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    print_symbol_occurrences(&tree, dialect, &args.symbol, args.output)
}

pub(super) fn symbol_report(args: SymbolReportArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        let outline = tree.outline(|head| dialect.is_definition_head(head));
        let occurrences = matching_symbol_occurrences(&tree, &args.symbol)
            .into_iter()
            .map(|occurrence| SymbolReportOccurrence {
                path: occurrence.path.to_string(),
                span: occurrence.span,
                context: outline
                    .iter()
                    .filter(|entry| span_contains(entry.span, occurrence.span))
                    .min_by_key(|entry| entry.span.end().get() - entry.span.start().get())
                    .map(|entry| SymbolOccurrenceContext {
                        path: entry.path.to_string(),
                        span: entry.span,
                        head: entry.head.clone(),
                        definition_like: entry.definition_like,
                    }),
            })
            .collect::<Vec<_>>();

        reports.push(SymbolReportFile {
            path: file.clone(),
            dialect,
            occurrences,
        });
    }

    print_symbol_report(&reports, &args.symbol, args.output)
}

fn print_symbol_occurrences(
    tree: &SyntaxTree,
    dialect: Dialect,
    symbol: &SymbolName,
    output: OutputFormat,
) -> Result<()> {
    let occurrences = matching_symbol_occurrences(tree, symbol);
    match output {
        OutputFormat::Text => {
            println!("dialect\t{}", dialect.label());
            for occurrence in occurrences {
                println!(
                    "{}\t{}..{}\t{}",
                    occurrence.path,
                    occurrence.span.start().get(),
                    occurrence.span.end().get(),
                    occurrence.text
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "dialect": dialect.label(),
                "symbol": symbol.as_str(),
                "occurrences": occurrences
                    .into_iter()
                    .map(|occurrence| json!({
                        "path": occurrence.path.to_string(),
                        "span": {
                            "start": occurrence.span.start().get(),
                            "end": occurrence.span.end().get(),
                        },
                        "text": occurrence.text,
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }
    Ok(())
}

fn print_symbol_report(
    reports: &[SymbolReportFile],
    symbol: &SymbolName,
    output: OutputFormat,
) -> Result<()> {
    let total_count = reports
        .iter()
        .map(|report| report.occurrences.len())
        .sum::<usize>();

    match output {
        OutputFormat::Text => {
            println!("symbol\t{symbol}");
            println!("files\t{}", reports.len());
            println!("total_count\t{total_count}");
            for report in reports {
                println!(
                    "{}\t{}\tcount={}",
                    report.path.display(),
                    report.dialect.label(),
                    report.occurrences.len()
                );
                for occurrence in &report.occurrences {
                    let context = occurrence
                        .context
                        .as_ref()
                        .map(|context| context.path.as_str())
                        .unwrap_or("<none>");
                    println!(
                        "\t{}\t{}..{}\tcontext={}",
                        occurrence.path,
                        occurrence.span.start().get(),
                        occurrence.span.end().get(),
                        context
                    );
                }
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "symbol": symbol.as_str(),
                "file_count": reports.len(),
                "total_count": total_count,
                "files": reports
                    .iter()
                    .map(|report| json!({
                        "path": report.path.display().to_string(),
                        "dialect": report.dialect.label(),
                        "count": report.occurrences.len(),
                        "occurrences": report
                            .occurrences
                            .iter()
                            .map(|occurrence| json!({
                                "path": occurrence.path.as_str(),
                                "span": {
                                    "start": occurrence.span.start().get(),
                                    "end": occurrence.span.end().get(),
                                },
                                "context": occurrence.context.as_ref().map(|context| json!({
                                    "path": context.path.as_str(),
                                    "span": {
                                        "start": context.span.start().get(),
                                        "end": context.span.end().get(),
                                    },
                                    "head": context.head.as_deref(),
                                    "definitionLike": context.definition_like,
                                })),
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }
    Ok(())
}

fn span_contains(outer: ByteSpan, inner: ByteSpan) -> bool {
    outer.start().get() <= inner.start().get() && inner.end().get() <= outer.end().get()
}
