use crate::domain::dialect::Dialect;
use crate::domain::sexpr::SyntaxTree;
use anyhow::Result;
use serde_json::json;

use crate::presentation::cli::args::OutputFormat;

pub(super) fn print_dialect(dialect: Dialect, output: OutputFormat) -> Result<()> {
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

pub(super) fn print_stats(tree: &SyntaxTree, dialect: Dialect, output: OutputFormat) -> Result<()> {
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

pub(super) fn print_agent_report(
    tree: &SyntaxTree,
    dialect: Dialect,
    output: OutputFormat,
) -> Result<()> {
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

pub(super) fn print_outline(
    tree: &SyntaxTree,
    dialect: Dialect,
    output: OutputFormat,
) -> Result<()> {
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
