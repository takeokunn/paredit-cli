use anyhow::{Context, Result};

use crate::domain::sexpr::{Edit, Formatter, SyntaxTree};
use crate::presentation::cli::args::{FormatArgs, ReplaceArgs, TargetArgs};
use crate::presentation::cli::shared::{detect_dialect, edit_target, read_input, resolve_target};

pub(in crate::presentation::cli) fn format(args: FormatArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let _dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    print!("{}", Formatter::new(args.indent).format(&tree));
    Ok(())
}

pub(in crate::presentation::cli) fn select(args: TargetArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    print!("{}", selection.text(&input.text));
    Ok(())
}

pub(in crate::presentation::cli) fn replace(args: ReplaceArgs) -> Result<()> {
    let input = read_input(args.file)?;
    SyntaxTree::parse(&args.with).context("replacement is not a valid S-expression document")?;
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    print!("{}", Edit::replace(&input.text, selection, &args.with));
    Ok(())
}

pub(in crate::presentation::cli) fn kill(args: TargetArgs) -> Result<()> {
    edit_target(args, Edit::kill)
}

pub(in crate::presentation::cli) fn wrap(args: TargetArgs) -> Result<()> {
    edit_target(args, Edit::wrap)
}

pub(in crate::presentation::cli) fn splice(args: TargetArgs) -> Result<()> {
    edit_target(args, Edit::splice)
}

pub(in crate::presentation::cli) fn raise(args: TargetArgs) -> Result<()> {
    edit_target(args, Edit::raise)
}

pub(in crate::presentation::cli) fn slurp_forward(args: TargetArgs) -> Result<()> {
    edit_target(args, Edit::slurp_forward)
}

pub(in crate::presentation::cli) fn slurp_backward(args: TargetArgs) -> Result<()> {
    edit_target(args, Edit::slurp_backward)
}

pub(in crate::presentation::cli) fn barf_forward(args: TargetArgs) -> Result<()> {
    edit_target(args, Edit::barf_forward)
}

pub(in crate::presentation::cli) fn barf_backward(args: TargetArgs) -> Result<()> {
    edit_target(args, Edit::barf_backward)
}
