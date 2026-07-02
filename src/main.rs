mod sexpr;

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use sexpr::{Edit, Formatter, Path, Selection, SyntaxTree};

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
struct FormatArgs {
    /// Input file. Reads stdin when omitted.
    #[arg(short, long)]
    file: Option<PathBuf>,
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Check(args) => {
            let input = read_input(args.file)?;
            SyntaxTree::parse(&input)?;
            println!("ok");
        }
        Command::Format(args) => {
            let input = read_input(args.file)?;
            let tree = SyntaxTree::parse(&input)?;
            print!("{}", Formatter::new(args.indent).format(&tree));
        }
        Command::Select(args) => {
            let input = read_input(args.file)?;
            let tree = SyntaxTree::parse(&input)?;
            let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
            print!("{}", selection.text(&input));
        }
        Command::Replace(args) => {
            let input = read_input(args.file)?;
            SyntaxTree::parse(&args.with)
                .context("replacement is not a valid S-expression document")?;
            let tree = SyntaxTree::parse(&input)?;
            let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
            print!("{}", Edit::replace(&input, selection, &args.with));
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

fn edit_target(
    args: TargetArgs,
    f: fn(&str, &SyntaxTree, Selection) -> Result<String>,
) -> Result<()> {
    let input = read_input(args.file)?;
    let tree = SyntaxTree::parse(&input)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    print!("{}", f(&input, &tree, selection)?);
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

fn read_input(file: Option<PathBuf>) -> Result<String> {
    match file {
        Some(path) => {
            fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))
        }
        None => {
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .context("failed to read stdin")?;
            Ok(input)
        }
    }
}
