use super::*;
use crate::application::form_report::{FormReport, FormReportRequest, build_form_report};

pub(super) fn form_report(args: FormReportArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    let report = build_form_report(FormReportRequest {
        input: &input.text,
        dialect,
        path: args.path,
        target: selection.view(),
        include_source: args.include_source,
    })?;

    print_form_report(&report, args.output)
}

fn print_form_report(report: &FormReport, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("dialect\t{}", report.dialect.label());
            if let Some(path) = &report.path {
                println!("path\t{path}");
            }
            println!(
                "span\t{}..{}",
                report.span.start().get(),
                report.span.end().get()
            );
            println!("kind\t{}", report.kind.label());
            println!(
                "delimiter\t{}",
                report.delimiter.map(delimiter_label).unwrap_or("<none>")
            );
            println!("head\t{}", report.head.as_deref().unwrap_or("<none>"));
            println!("definition_like\t{}", report.definition_like);
            println!("child_count\t{}", report.child_count);
            println!("atom_count\t{}", report.atom_count);
            println!("list_count\t{}", report.list_count);
            println!("max_depth\t{}", report.max_depth);
            for symbol in &report.symbols {
                println!(
                    "symbol\t{}\tcount={}\tfirst_span={}..{}",
                    symbol.symbol,
                    symbol.count,
                    symbol.first_span.start().get(),
                    symbol.first_span.end().get()
                );
            }
            if let Some(source) = &report.source {
                println!("source\t{source}");
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "dialect": report.dialect.label(),
                "path": report.path.as_ref().map(ToString::to_string),
                "span": {
                    "start": report.span.start().get(),
                    "end": report.span.end().get(),
                },
                "kind": report.kind.label(),
                "delimiter": report.delimiter.map(delimiter_label),
                "head": report.head.as_deref(),
                "definitionLike": report.definition_like,
                "childCount": report.child_count,
                "atomCount": report.atom_count,
                "listCount": report.list_count,
                "maxDepth": report.max_depth,
                "symbols": report
                    .symbols
                    .iter()
                    .map(|symbol| json!({
                        "symbol": symbol.symbol.as_str(),
                        "count": symbol.count,
                        "firstSpan": {
                            "start": symbol.first_span.start().get(),
                            "end": symbol.first_span.end().get(),
                        },
                    }))
                    .collect::<Vec<_>>(),
                "source": report.source.as_deref(),
            }))?
        ),
    }

    Ok(())
}

fn delimiter_label(delimiter: Delimiter) -> &'static str {
    match delimiter {
        Delimiter::Paren => "paren",
        Delimiter::Bracket => "bracket",
        Delimiter::Brace => "brace",
    }
}
