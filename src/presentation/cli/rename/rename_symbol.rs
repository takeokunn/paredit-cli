use anyhow::Result;

use super::super::{detect_dialect, read_input};
use super::args::RenameSymbolArgs;
use super::render::symbol::print_rename_plan;
use crate::domain::sexpr::SyntaxTree;

pub(in crate::presentation::cli) fn rename_symbol(args: RenameSymbolArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    if args.plan {
        print_rename_plan(&tree, dialect, &args.from, &args.to, args.output)?;
    } else {
        print!("{}", tree.rename_symbol(&input.text, &args.from, &args.to));
    }
    Ok(())
}
