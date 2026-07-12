use anyhow::Result;

use super::super::read_input_dialect_and_tree;
use super::args::RenameSymbolArgs;
use super::render::symbol::print_rename_plan;

pub(in crate::presentation::cli) fn rename_symbol(args: RenameSymbolArgs) -> Result<()> {
    let (input, dialect, tree) = read_input_dialect_and_tree(args.file, args.dialect)?;
    if args.plan {
        print_rename_plan(&tree, dialect, &args.from, &args.to, args.output)?;
    } else {
        print!("{}", tree.rename_symbol(&input.text, &args.from, &args.to));
    }
    Ok(())
}
