use anyhow::Result;

use crate::presentation::cli::args::{AnalyzeArgs, InputArgs};
use crate::presentation::cli::shared::read_input_dialect_and_tree;

use super::render::{print_agent_report, print_dialect, print_outline, print_stats};

pub(in crate::presentation::cli) fn check(args: InputArgs) -> Result<()> {
    let (_, _, _tree) = read_input_dialect_and_tree(args.file, None)?;
    println!("ok");
    Ok(())
}

pub(in crate::presentation::cli) fn dialect(args: AnalyzeArgs) -> Result<()> {
    let (_, dialect, _) = read_input_dialect_and_tree(args.file, args.dialect)?;
    print_dialect(dialect, args.output)
}

pub(in crate::presentation::cli) fn stats(args: AnalyzeArgs) -> Result<()> {
    let (_, dialect, tree) = read_input_dialect_and_tree(args.file, args.dialect)?;
    print_stats(&tree, dialect, args.output)
}

pub(in crate::presentation::cli) fn agent_report(args: AnalyzeArgs) -> Result<()> {
    let (_, dialect, tree) = read_input_dialect_and_tree(args.file, args.dialect)?;
    print_agent_report(&tree, dialect, args.output)
}

pub(in crate::presentation::cli) fn outline(args: AnalyzeArgs) -> Result<()> {
    let (_, dialect, tree) = read_input_dialect_and_tree(args.file, args.dialect)?;
    print_outline(&tree, dialect, args.output)
}
