use anyhow::{Context, Result, bail};

use crate::domain::sexpr::{Edit, Formatter, SyntaxTree};
use crate::presentation::cli::args::{
    EditTargetArgs, FormatArgs, RepairArgs, ReplaceArgs, TargetArgs,
};
use crate::presentation::cli::shared::{
    edit_target, emit_document, read_input, read_input_dialect_and_tree, resolve_target,
};

pub(in crate::presentation::cli) fn format(args: FormatArgs) -> Result<()> {
    let (input, _, tree) = read_input_dialect_and_tree(args.file, args.dialect)?;
    let rendered = Formatter::new(args.indent).format(&tree);
    emit_document(&input, args.write, args.diff, rendered)
}

pub(in crate::presentation::cli) fn repair_unclosed_lists(args: RepairArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let repaired = SyntaxTree::repair_unclosed_lists(&input.text)
        .context("repair-unclosed-lists only repairs unclosed lists")?;
    if repaired == input.text {
        bail!("input is already balanced");
    }
    emit_document(&input, args.write, args.diff, repaired)
}

pub(in crate::presentation::cli) fn select(args: TargetArgs) -> Result<()> {
    let (input, _, tree) = read_input_dialect_and_tree(args.file, None)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    print!("{}", selection.text(&input.text));
    Ok(())
}

pub(in crate::presentation::cli) fn replace(args: ReplaceArgs) -> Result<()> {
    let (input, _, tree) = read_input_dialect_and_tree(args.file, None)?;
    SyntaxTree::parse(&args.with).context("replacement is not a valid S-expression document")?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    let rewritten = Edit::replace(&input.text, selection, &args.with);
    let rewritten = Edit::normalize_changed_line_trivia(&input.text, rewritten)?;
    emit_document(&input, args.write, args.diff, rewritten)
}

pub(in crate::presentation::cli) fn kill(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::kill)
}

pub(in crate::presentation::cli) fn wrap(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::wrap)
}

pub(in crate::presentation::cli) fn splice(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::splice)
}

pub(in crate::presentation::cli) fn raise(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::raise)
}

pub(in crate::presentation::cli) fn transpose_forward(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::transpose_forward)
}

pub(in crate::presentation::cli) fn transpose_backward(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::transpose_backward)
}

pub(in crate::presentation::cli) fn slurp_forward(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::slurp_forward)
}

pub(in crate::presentation::cli) fn slurp_backward(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::slurp_backward)
}

pub(in crate::presentation::cli) fn barf_forward(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::barf_forward)
}

pub(in crate::presentation::cli) fn barf_backward(args: EditTargetArgs) -> Result<()> {
    edit_target(args, Edit::barf_backward)
}
