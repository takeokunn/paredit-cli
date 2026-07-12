use anyhow::{Context, Result};
use serde_json::json;

use crate::domain::sexpr::SyntaxTree;
use crate::presentation::cli::args::{AnalyzeArgs, OutputFormat};
use crate::presentation::cli::shared::{read_input_and_dialect, read_input_dialect_and_tree};

use super::render::{print_agent_report, print_dialect, print_outline, print_stats};

pub(in crate::presentation::cli) fn check(args: AnalyzeArgs) -> Result<()> {
    match args.output {
        OutputFormat::Text => {
            read_input_dialect_and_tree(args.file, args.dialect)?;
            println!("ok");
            Ok(())
        }
        OutputFormat::Json => {
            let (input, dialect) = read_input_and_dialect(args.file, args.dialect)?;
            let file = input.file.as_deref().map(|path| path.display().to_string());
            let parse_error = SyntaxTree::parse(&input.text).err();
            let report = json!({
                "schema_version": 1,
                "status": if parse_error.is_none() { "ok" } else { "error" },
                "file": file,
                "dialect": dialect.label(),
                "error": parse_error.as_ref().map(ToString::to_string),
            });
            println!("{}", serde_json::to_string_pretty(&report)?);
            match parse_error {
                None => Ok(()),
                Some(error) => Err(error).context("input is not a balanced S-expression document"),
            }
        }
    }
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
