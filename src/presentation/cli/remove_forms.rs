use super::*;
use crate::sexpr::Edit;

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  paredit refactor remove-forms --path 0 < forms.lisp\n  paredit refactor remove-forms --file forms.lisp --path 0 --write --output json"
)]
pub(super) struct RemoveFormsArgs {
    /// Input file. Required when --write is used; reads stdin otherwise.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Selected form path. Pass repeatedly, for example --path 0 --path 3.
    #[arg(long = "path", required = true)]
    paths: Vec<Path>,
    /// Rewrite the input file in place. Without this flag, only prints a plan.
    #[arg(long)]
    write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

pub(super) fn remove_forms(args: RemoveFormsArgs) -> Result<()> {
    let RemoveFormsArgs {
        file,
        dialect,
        paths,
        write,
        output,
    } = args;

    if write && file.is_none() {
        anyhow::bail!("remove-forms --write requires --file");
    }

    let input = read_input(file.clone())?;
    let dialect = detect_dialect(&input, dialect);
    let tree = SyntaxTree::parse(&input.text)?;

    let mut selections = Vec::with_capacity(paths.len());
    let mut targets = Vec::with_capacity(paths.len());
    for path in paths {
        let selection = tree.select_path(&path)?;
        let removal_span = Edit::removal_span(&input.text, &tree, selection);
        targets.push(RemoveFormTarget {
            path,
            span: selection.span(),
            removal_span,
            text: selection.text(&input.text).to_owned(),
        });
        selections.push(selection);
    }

    let rewritten = Edit::kill_many(&input.text, &tree, &selections)?;
    let changed = rewritten != input.text;
    let written = write && changed;
    if written {
        let Some(path) = file.as_ref() else {
            anyhow::bail!("remove-forms --write requires --file");
        };
        write_file_with_rollback(path.clone(), rewritten.clone())?;
    }

    print_remove_forms_plan(
        file.as_ref(),
        dialect,
        &targets,
        changed,
        written,
        &rewritten,
        output,
    )
}

struct RemoveFormTarget {
    path: Path,
    span: ByteSpan,
    removal_span: ByteSpan,
    text: String,
}

fn print_remove_forms_plan(
    path: Option<&PathBuf>,
    dialect: Dialect,
    targets: &[RemoveFormTarget],
    changed: bool,
    written: bool,
    rewritten: &str,
    output: OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!(
                "file\t{}",
                path.map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<stdin>".to_owned())
            );
            println!("dialect\t{}", dialect.label());
            println!("path_count\t{}", targets.len());
            println!("changed\t{}", changed);
            println!("written\t{}", written);
            for target in targets {
                println!(
                    "target\t{}\t{}..{}\t{}..{}\t{}",
                    target.path,
                    target.span.start().get(),
                    target.span.end().get(),
                    target.removal_span.start().get(),
                    target.removal_span.end().get(),
                    target.text.as_str()
                );
            }
            println!("rewritten\n{}", rewritten);
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "file": path.map(|path| path.display().to_string()),
                "dialect": dialect.label(),
                "path_count": targets.len(),
                "changed": changed,
                "written": written,
                "targets": targets
                    .iter()
                    .map(|target| json!({
                        "path": target.path.to_string(),
                        "span": {
                            "start": target.span.start().get(),
                            "end": target.span.end().get(),
                        },
                        "removal_span": {
                            "start": target.removal_span.start().get(),
                            "end": target.removal_span.end().get(),
                        },
                        "text": target.text.as_str(),
                    }))
                    .collect::<Vec<_>>(),
                "rewritten": rewritten,
            }))?
        ),
    }

    Ok(())
}
