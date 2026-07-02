use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use paredit_cli::dialect::Dialect;
use paredit_cli::sexpr::{Edit, Formatter, Path, Selection, SymbolName, SyntaxTree};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate that input is a balanced S-expression document.
    Check(InputArgs),
    /// Detect Lisp dialect from --file extension or explicit --dialect.
    Dialect(AnalyzeArgs),
    /// Print parse, dialect, and structural metrics for agent planning.
    Stats(AnalyzeArgs),
    /// Print a complete JSON report for AI coding agent refactor planning.
    AgentReport(AnalyzeArgs),
    /// Print top-level forms with paths, spans, and definition hints.
    Outline(AnalyzeArgs),
    /// Find exact atom occurrences without touching strings or comments.
    FindSymbol(SymbolQueryArgs),
    /// Rename exact atom occurrences without touching strings or comments.
    RenameSymbol(RenameSymbolArgs),
    /// Plan or apply an exact atom rename across explicit files.
    RenameSymbols(RenameSymbolsArgs),
    /// Extract the selected expression into a zero-argument top-level function.
    ExtractFunction(ExtractFunctionArgs),
    /// Replace the selected expression with a local binding in the enclosing list.
    IntroduceLet(IntroduceLetArgs),
    /// Print a canonical, indentation-based rendering.
    Format(FormatArgs),
    /// Print the S-expression selected by --path or --at.
    Select(TargetArgs),
    /// Replace the selected S-expression with replacement text.
    Replace(ReplaceArgs),
    /// Remove the selected S-expression.
    Kill(TargetArgs),
    /// Wrap the selected S-expression in a new list.
    Wrap(TargetArgs),
    /// Remove one list pair while keeping its children.
    Splice(TargetArgs),
    /// Replace the selected expression's parent list with the selected expression.
    Raise(TargetArgs),
    /// Pull the next sibling into the selected list.
    SlurpForward(TargetArgs),
    /// Pull the previous sibling into the selected list.
    SlurpBackward(TargetArgs),
    /// Push the last child out of the selected list.
    BarfForward(TargetArgs),
    /// Push the first child out of the selected list.
    BarfBackward(TargetArgs),
}

#[derive(Debug, Args)]
struct InputArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    file: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct AnalyzeArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

#[derive(Debug, Args)]
struct SymbolQueryArgs {
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
struct RenameSymbolArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Exact source symbol atom.
    #[arg(long)]
    from: SymbolName,
    /// Exact replacement symbol atom.
    #[arg(long)]
    to: SymbolName,
    /// Print occurrence metadata instead of rewritten source.
    #[arg(long)]
    plan: bool,
    /// Output format for --plan.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

#[derive(Debug, Args)]
struct RenameSymbolsArgs {
    /// Files to scan and optionally rewrite.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Exact source symbol atom.
    #[arg(long)]
    from: SymbolName,
    /// Exact replacement symbol atom.
    #[arg(long)]
    to: SymbolName,
    /// Rewrite changed files in place. Without this flag, only prints a plan.
    #[arg(long)]
    write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

#[derive(Debug, Args)]
struct ExtractFunctionArgs {
    /// Input file. Required when --write is used; reads stdin otherwise.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Select by child index path, for example 0.2.1.
    #[arg(long, conflicts_with = "at")]
    path: Option<Path>,
    /// Select the smallest expression containing byte offset.
    #[arg(long, conflicts_with = "path")]
    at: Option<usize>,
    /// New top-level function name.
    #[arg(long)]
    name: SymbolName,
    /// Rewrite the input file in place. Without this flag, only prints a plan.
    #[arg(long)]
    write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

#[derive(Debug, Args)]
struct IntroduceLetArgs {
    /// Input file. Required when --write is used; reads stdin otherwise.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Select by child index path, for example 0.2.1.
    #[arg(long, conflicts_with = "at")]
    path: Option<Path>,
    /// Select the smallest expression containing byte offset.
    #[arg(long, conflicts_with = "path")]
    at: Option<usize>,
    /// New local binding name.
    #[arg(long)]
    name: SymbolName,
    /// Rewrite the input file in place. Without this flag, only prints a plan.
    #[arg(long)]
    write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

#[derive(Debug, Args)]
struct FormatArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Number of spaces per nesting level.
    #[arg(long, default_value_t = 2)]
    indent: usize,
}

#[derive(Debug, Args)]
struct TargetArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Select by child index path, for example 0.2.1.
    #[arg(long, conflicts_with = "at")]
    path: Option<Path>,
    /// Select the smallest expression containing byte offset.
    #[arg(long, conflicts_with = "path")]
    at: Option<usize>,
}

#[derive(Debug, Args)]
struct ReplaceArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Select by child index path, for example 0.2.1.
    #[arg(long, conflicts_with = "at")]
    path: Option<Path>,
    /// Select the smallest expression containing byte offset.
    #[arg(long, conflicts_with = "path")]
    at: Option<usize>,
    /// Replacement S-expression text.
    #[arg(long)]
    with: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DialectArg {
    CommonLisp,
    EmacsLisp,
    Scheme,
    Clojure,
    Janet,
    Fennel,
    Unknown,
}

impl From<DialectArg> for Dialect {
    fn from(value: DialectArg) -> Self {
        match value {
            DialectArg::CommonLisp => Self::CommonLisp,
            DialectArg::EmacsLisp => Self::EmacsLisp,
            DialectArg::Scheme => Self::Scheme,
            DialectArg::Clojure => Self::Clojure,
            DialectArg::Janet => Self::Janet,
            DialectArg::Fennel => Self::Fennel,
            DialectArg::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug)]
struct SourceInput {
    text: String,
    file: Option<PathBuf>,
}

#[derive(Debug)]
struct RenameFileReport {
    path: PathBuf,
    dialect: Dialect,
    count: usize,
    changed: bool,
    written: bool,
}

#[derive(Debug)]
struct ExtractFunctionPlan {
    dialect: Dialect,
    path: Option<Path>,
    span_start: usize,
    span_end: usize,
    name: SymbolName,
    call: String,
    definition: String,
    rewritten: String,
    changed: bool,
    written: bool,
}

#[derive(Debug)]
struct IntroduceLetPlan {
    dialect: Dialect,
    path: Option<Path>,
    selected_span_start: usize,
    selected_span_end: usize,
    enclosing_span_start: usize,
    enclosing_span_end: usize,
    name: SymbolName,
    binding_value: String,
    replacement: String,
    rewritten: String,
    changed: bool,
    written: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Check(args) => {
            let input = read_input(args.file)?;
            SyntaxTree::parse(&input.text)?;
            println!("ok");
        }
        Command::Dialect(args) => {
            let input = read_input(args.file)?;
            let dialect = detect_dialect(&input, args.dialect);
            print_dialect(dialect, args.output)?;
        }
        Command::Stats(args) => {
            let input = read_input(args.file)?;
            let dialect = detect_dialect(&input, args.dialect);
            let tree = SyntaxTree::parse(&input.text)?;
            print_stats(&tree, dialect, args.output)?;
        }
        Command::AgentReport(args) => {
            let input = read_input(args.file)?;
            let dialect = detect_dialect(&input, args.dialect);
            let tree = SyntaxTree::parse(&input.text)?;
            print_agent_report(&tree, dialect, args.output)?;
        }
        Command::Outline(args) => {
            let input = read_input(args.file)?;
            let dialect = detect_dialect(&input, args.dialect);
            let tree = SyntaxTree::parse(&input.text)?;
            print_outline(&tree, dialect, args.output)?;
        }
        Command::FindSymbol(args) => {
            let input = read_input(args.file)?;
            let dialect = detect_dialect(&input, args.dialect);
            let tree = SyntaxTree::parse(&input.text)?;
            print_symbol_occurrences(&tree, dialect, &args.symbol, args.output)?;
        }
        Command::RenameSymbol(args) => {
            let input = read_input(args.file)?;
            let dialect = detect_dialect(&input, args.dialect);
            let tree = SyntaxTree::parse(&input.text)?;
            if args.plan {
                print_rename_plan(&tree, dialect, &args.from, &args.to, args.output)?;
            } else {
                print!("{}", tree.rename_symbol(&input.text, &args.from, &args.to));
            }
        }
        Command::RenameSymbols(args) => rename_symbols(args)?,
        Command::ExtractFunction(args) => extract_function(args)?,
        Command::IntroduceLet(args) => introduce_let(args)?,
        Command::Format(args) => {
            let input = read_input(args.file)?;
            let _dialect = detect_dialect(&input, args.dialect);
            let tree = SyntaxTree::parse(&input.text)?;
            print!("{}", Formatter::new(args.indent).format(&tree));
        }
        Command::Select(args) => {
            let input = read_input(args.file)?;
            let tree = SyntaxTree::parse(&input.text)?;
            let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
            print!("{}", selection.text(&input.text));
        }
        Command::Replace(args) => {
            let input = read_input(args.file)?;
            SyntaxTree::parse(&args.with)
                .context("replacement is not a valid S-expression document")?;
            let tree = SyntaxTree::parse(&input.text)?;
            let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
            print!("{}", Edit::replace(&input.text, selection, &args.with));
        }
        Command::Kill(args) => edit_target(args, Edit::kill)?,
        Command::Wrap(args) => edit_target(args, Edit::wrap)?,
        Command::Splice(args) => edit_target(args, Edit::splice)?,
        Command::Raise(args) => edit_target(args, Edit::raise)?,
        Command::SlurpForward(args) => edit_target(args, Edit::slurp_forward)?,
        Command::SlurpBackward(args) => edit_target(args, Edit::slurp_backward)?,
        Command::BarfForward(args) => edit_target(args, Edit::barf_forward)?,
        Command::BarfBackward(args) => edit_target(args, Edit::barf_backward)?,
    }
    Ok(())
}

fn extract_function(args: ExtractFunctionArgs) -> Result<()> {
    if args.write && args.file.is_none() {
        anyhow::bail!("--write requires --file");
    }

    let input = read_input(args.file.clone())?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    let span = selection.span();
    let selected = selection.text(&input.text).to_owned();
    let call = extracted_call(dialect, &args.name);
    let definition = extracted_definition(dialect, &args.name, &selected);
    let replaced = Edit::replace(&input.text, selection, &call);
    let rewritten = append_top_level_definition(&replaced, &definition);

    SyntaxTree::parse(&rewritten)
        .context("extracted output is not a valid S-expression document")?;

    let changed = rewritten != input.text;
    let written = args.write && changed;
    if written {
        let file = input
            .file
            .as_ref()
            .expect("--write was validated to require --file");
        fs::write(file, &rewritten)
            .with_context(|| format!("failed to write {}", file.display()))?;
    }

    let plan = ExtractFunctionPlan {
        dialect,
        path: args.path,
        span_start: span.start().get(),
        span_end: span.end().get(),
        name: args.name,
        call,
        definition,
        rewritten,
        changed,
        written,
    };
    print_extract_function_plan(&plan, args.output)
}

fn extracted_call(_dialect: Dialect, name: &SymbolName) -> String {
    format!("({})", name.as_str())
}

fn extracted_definition(dialect: Dialect, name: &SymbolName, body: &str) -> String {
    match dialect {
        Dialect::Scheme => format!("(define ({}) {})", name.as_str(), body),
        Dialect::Clojure | Dialect::Janet => format!("(defn {} [] {})", name.as_str(), body),
        Dialect::Fennel => format!("(fn {} [] {})", name.as_str(), body),
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Unknown => {
            format!("(defun {} () {})", name.as_str(), body)
        }
    }
}

fn append_top_level_definition(input: &str, definition: &str) -> String {
    let mut output = input.trim_end().to_owned();
    if !output.is_empty() {
        output.push_str("\n\n");
    }
    output.push_str(definition);
    output.push('\n');
    output
}

fn print_extract_function_plan(plan: &ExtractFunctionPlan, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("dialect\t{}", plan.dialect.label());
            if let Some(path) = &plan.path {
                println!("path\t{path}");
            }
            println!("span\t{}..{}", plan.span_start, plan.span_end);
            println!("name\t{}", plan.name);
            println!("call\t{}", plan.call);
            println!("definition\t{}", plan.definition);
            println!("changed\t{}", plan.changed);
            println!("written\t{}", plan.written);
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "dialect": plan.dialect.label(),
                "path": plan.path.as_ref().map(ToString::to_string),
                "span": {
                    "start": plan.span_start,
                    "end": plan.span_end,
                },
                "name": plan.name.as_str(),
                "call": plan.call,
                "definition": plan.definition,
                "changed": plan.changed,
                "written": plan.written,
                "rewritten": plan.rewritten,
            }))?
        ),
    }
    Ok(())
}

fn introduce_let(args: IntroduceLetArgs) -> Result<()> {
    if args.write && args.file.is_none() {
        anyhow::bail!("--write requires --file");
    }

    let input = read_input(args.file.clone())?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    let selected_span = selection.span();
    let enclosing_span = selection.enclosing_list_span()?;
    let selected = selected_span.slice(&input.text).to_owned();
    let enclosing = enclosing_span.slice(&input.text);
    let relative_start = selected_span.start().get() - enclosing_span.start().get();
    let relative_end = selected_span.end().get() - enclosing_span.start().get();
    let mut enclosed_replacement =
        String::with_capacity(enclosing.len() + args.name.as_str().len());
    enclosed_replacement.push_str(&enclosing[..relative_start]);
    enclosed_replacement.push_str(args.name.as_str());
    enclosed_replacement.push_str(&enclosing[relative_end..]);
    let replacement = introduced_let(dialect, &args.name, &selected, &enclosed_replacement);
    let rewritten = replace_span(&input.text, enclosing_span, &replacement);

    SyntaxTree::parse(&rewritten)
        .context("introduced-let output is not a valid S-expression document")?;

    let changed = rewritten != input.text;
    let written = args.write && changed;
    if written {
        let file = input
            .file
            .as_ref()
            .expect("--write was validated to require --file");
        fs::write(file, &rewritten)
            .with_context(|| format!("failed to write {}", file.display()))?;
    }

    let plan = IntroduceLetPlan {
        dialect,
        path: args.path,
        selected_span_start: selected_span.start().get(),
        selected_span_end: selected_span.end().get(),
        enclosing_span_start: enclosing_span.start().get(),
        enclosing_span_end: enclosing_span.end().get(),
        name: args.name,
        binding_value: selected,
        replacement,
        rewritten,
        changed,
        written,
    };
    print_introduce_let_plan(&plan, args.output)
}

fn introduced_let(dialect: Dialect, name: &SymbolName, value: &str, body: &str) -> String {
    match dialect {
        Dialect::Clojure | Dialect::Janet | Dialect::Fennel => {
            format!("(let [{} {}] {})", name.as_str(), value, body)
        }
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Scheme | Dialect::Unknown => {
            format!("(let (({} {})) {})", name.as_str(), value, body)
        }
    }
}

fn replace_span(input: &str, span: paredit_cli::sexpr::ByteSpan, replacement: &str) -> String {
    let mut output = String::with_capacity(input.len() - span.len() + replacement.len());
    output.push_str(&input[..span.start().get()]);
    output.push_str(replacement);
    output.push_str(&input[span.end().get()..]);
    output
}

fn print_introduce_let_plan(plan: &IntroduceLetPlan, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("dialect\t{}", plan.dialect.label());
            if let Some(path) = &plan.path {
                println!("path\t{path}");
            }
            println!(
                "selected_span\t{}..{}",
                plan.selected_span_start, plan.selected_span_end
            );
            println!(
                "enclosing_span\t{}..{}",
                plan.enclosing_span_start, plan.enclosing_span_end
            );
            println!("name\t{}", plan.name);
            println!("binding_value\t{}", plan.binding_value);
            println!("replacement\t{}", plan.replacement);
            println!("changed\t{}", plan.changed);
            println!("written\t{}", plan.written);
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "dialect": plan.dialect.label(),
                "path": plan.path.as_ref().map(ToString::to_string),
                "selected_span": {
                    "start": plan.selected_span_start,
                    "end": plan.selected_span_end,
                },
                "enclosing_span": {
                    "start": plan.enclosing_span_start,
                    "end": plan.enclosing_span_end,
                },
                "name": plan.name.as_str(),
                "binding_value": plan.binding_value,
                "replacement": plan.replacement,
                "changed": plan.changed,
                "written": plan.written,
                "rewritten": plan.rewritten,
            }))?
        ),
    }
    Ok(())
}

fn rename_symbols(args: RenameSymbolsArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        let count = matching_symbol_occurrences(&tree, &args.from).len();
        let rewritten = tree.rename_symbol(&input.text, &args.from, &args.to);
        let changed = rewritten != input.text;
        let written = args.write && changed;

        if written {
            SyntaxTree::parse(&rewritten)
                .with_context(|| format!("renamed output is invalid for {}", file.display()))?;
            fs::write(file, rewritten)
                .with_context(|| format!("failed to write {}", file.display()))?;
        }

        reports.push(RenameFileReport {
            path: file.clone(),
            dialect,
            count,
            changed,
            written,
        });
    }

    print_rename_symbols_report(&reports, &args.from, &args.to, args.write, args.output)
}

fn print_rename_symbols_report(
    reports: &[RenameFileReport],
    from: &SymbolName,
    to: &SymbolName,
    write: bool,
    output: OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("from\t{from}");
            println!("to\t{to}");
            println!("write\t{write}");
            for report in reports {
                println!(
                    "{}\t{}\tcount={}\tchanged={}\twritten={}",
                    report.path.display(),
                    report.dialect.label(),
                    report.count,
                    report.changed,
                    report.written
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "from": from.as_str(),
                "to": to.as_str(),
                "write": write,
                "files": reports
                    .iter()
                    .map(|report| json!({
                        "path": report.path.display().to_string(),
                        "dialect": report.dialect.label(),
                        "count": report.count,
                        "changed": report.changed,
                        "written": report.written,
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }
    Ok(())
}

fn print_dialect(dialect: Dialect, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Text => println!("{dialect}"),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "dialect": dialect.label(),
                "family": dialect.family(),
            }))?
        ),
    }
    Ok(())
}

fn print_stats(tree: &SyntaxTree, dialect: Dialect, output: OutputFormat) -> Result<()> {
    let atoms = tree.atom_occurrences();
    let outline = tree.outline(|head| dialect.is_definition_head(head));
    match output {
        OutputFormat::Text => {
            println!("dialect\t{}", dialect.label());
            println!("top_level_forms\t{}", tree.root_children().len());
            println!("outline_entries\t{}", outline.len());
            println!("atom_occurrences\t{}", atoms.len());
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "dialect": dialect.label(),
                "topLevelForms": tree.root_children().len(),
                "outlineEntries": outline.len(),
                "atomOccurrences": atoms.len(),
            }))?
        ),
    }
    Ok(())
}

fn print_agent_report(tree: &SyntaxTree, dialect: Dialect, output: OutputFormat) -> Result<()> {
    let atoms = tree.atom_occurrences();
    let outline = tree.outline(|head| dialect.is_definition_head(head));
    let payload = json!({
        "dialect": {
            "label": dialect.label(),
            "family": dialect.family(),
        },
        "metrics": {
            "topLevelForms": tree.root_children().len(),
            "outlineEntries": outline.len(),
            "atomOccurrences": atoms.len(),
        },
        "outline": outline
            .into_iter()
            .map(|entry| {
                json!({
                    "path": entry.path.to_string(),
                    "span": {
                        "start": entry.span.start().get(),
                        "end": entry.span.end().get(),
                    },
                    "head": entry.head,
                    "definitionLike": entry.definition_like,
                })
            })
            .collect::<Vec<_>>(),
        "atoms": atoms
            .into_iter()
            .map(|occurrence| {
                json!({
                    "path": occurrence.path.to_string(),
                    "span": {
                        "start": occurrence.span.start().get(),
                        "end": occurrence.span.end().get(),
                    },
                    "text": occurrence.text,
                })
            })
            .collect::<Vec<_>>(),
    });

    match output {
        OutputFormat::Text => {
            println!("dialect\t{}", dialect.label());
            println!("top_level_forms\t{}", tree.root_children().len());
            println!("use --output json for full outline and atom spans");
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&payload)?),
    }
    Ok(())
}

fn print_outline(tree: &SyntaxTree, dialect: Dialect, output: OutputFormat) -> Result<()> {
    let entries = tree.outline(|head| dialect.is_definition_head(head));
    match output {
        OutputFormat::Text => {
            for entry in entries {
                println!(
                    "{}\t{}..{}\t{}\t{}",
                    entry.path,
                    entry.span.start().get(),
                    entry.span.end().get(),
                    entry.head.as_deref().unwrap_or("<unknown>"),
                    entry.definition_like
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(
                &entries
                    .into_iter()
                    .map(|entry| {
                        json!({
                            "path": entry.path.to_string(),
                            "span": {
                                "start": entry.span.start().get(),
                                "end": entry.span.end().get(),
                            },
                            "head": entry.head,
                            "definitionLike": entry.definition_like,
                        })
                    })
                    .collect::<Vec<_>>()
            )?
        ),
    }
    Ok(())
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

fn print_rename_plan(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    output: OutputFormat,
) -> Result<()> {
    let occurrences = matching_symbol_occurrences(tree, from);
    match output {
        OutputFormat::Text => {
            println!("dialect\t{}", dialect.label());
            println!("from\t{from}");
            println!("to\t{to}");
            println!("count\t{}", occurrences.len());
            for occurrence in occurrences {
                println!(
                    "{}\t{}..{}",
                    occurrence.path,
                    occurrence.span.start().get(),
                    occurrence.span.end().get()
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "dialect": dialect.label(),
                "from": from.as_str(),
                "to": to.as_str(),
                "count": occurrences.len(),
                "occurrences": occurrences
                    .into_iter()
                    .map(|occurrence| json!({
                        "path": occurrence.path.to_string(),
                        "span": {
                            "start": occurrence.span.start().get(),
                            "end": occurrence.span.end().get(),
                        },
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }
    Ok(())
}

fn matching_symbol_occurrences(
    tree: &SyntaxTree,
    symbol: &SymbolName,
) -> Vec<paredit_cli::sexpr::AtomOccurrence> {
    tree.atom_occurrences()
        .into_iter()
        .filter(|occurrence| occurrence.text == symbol.as_str())
        .collect()
}

fn edit_target(
    args: TargetArgs,
    f: fn(&str, &SyntaxTree, Selection<'_>) -> Result<String>,
) -> Result<()> {
    let input = read_input(args.file)?;
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    print!("{}", f(&input.text, &tree, selection)?);
    Ok(())
}

fn resolve_target<'a>(
    tree: &'a SyntaxTree,
    path: Option<&Path>,
    at: Option<usize>,
) -> Result<Selection<'a>> {
    match (path, at) {
        (Some(path), None) => tree.select_path(path),
        (None, Some(offset)) => tree.select_at(offset),
        (None, None) => anyhow::bail!("target required: pass --path or --at"),
        (Some(_), Some(_)) => anyhow::bail!("pass only one of --path or --at"),
    }
}

fn detect_dialect(input: &SourceInput, explicit: Option<DialectArg>) -> Dialect {
    Dialect::detect(input.file.as_deref(), explicit.map(Into::into))
}

fn read_input(file: Option<PathBuf>) -> Result<SourceInput> {
    match file {
        Some(path) => {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            Ok(SourceInput {
                text,
                file: Some(path),
            })
        }
        None => {
            let mut text = String::new();
            io::stdin()
                .read_to_string(&mut text)
                .context("failed to read stdin")?;
            Ok(SourceInput { text, file: None })
        }
    }
}
