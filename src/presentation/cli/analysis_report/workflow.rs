use crate::domain::common_lisp::{
    common_lisp_reader_escape_diagnostics, function_value_namespace_diagnostics,
};
use crate::domain::sexpr::SyntaxTree;
use anyhow::{bail, Result};

use crate::presentation::cli::args::{AnalyzeArgs, InputArgs};
use crate::presentation::cli::{detect_dialect, read_input};

use super::render::{print_agent_report, print_dialect, print_outline, print_stats};

pub(in crate::presentation::cli) fn check(args: InputArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let tree = SyntaxTree::parse(&input.text)?;
    let dialect = detect_dialect(&input, None);
    if let Some(diagnostic) = common_lisp_reader_escape_diagnostics(&input.text, dialect)
        .into_iter()
        .next()
    {
        bail!(
            "{}: {}. {}",
            diagnostic.code(),
            diagnostic.message(),
            diagnostic.suggestion()
        );
    }
    if let Some(diagnostic) = function_value_namespace_diagnostics(&tree, dialect)?
        .into_iter()
        .next()
    {
        bail!(
            "{}: {}. {}",
            diagnostic.code(),
            diagnostic.message(),
            diagnostic.suggestion()
        );
    }
    println!("ok");
    Ok(())
}

pub(in crate::presentation::cli) fn dialect(args: AnalyzeArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let dialect = detect_dialect(&input, args.dialect);
    print_dialect(dialect, args.output)
}

pub(in crate::presentation::cli) fn stats(args: AnalyzeArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    print_stats(&tree, dialect, args.output)
}

pub(in crate::presentation::cli) fn agent_report(args: AnalyzeArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    print_agent_report(&tree, dialect, args.output)
}

pub(in crate::presentation::cli) fn outline(args: AnalyzeArgs) -> Result<()> {
    let input = read_input(args.file)?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    print_outline(&tree, dialect, args.output)
}
