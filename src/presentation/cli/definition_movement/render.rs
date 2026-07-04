use anyhow::Result;
use serde_json::json;

use crate::application::usecase::sort_definitions::SortDefinitionsPlan;
use crate::application::usecase::split_file::SplitFilePlan;

use super::super::OutputFormat;
use super::types::{MoveDefinitionPlan, MoveFormPlan};

pub(super) fn print_sort_definitions_plan(
    plan: &SortDefinitionsPlan,
    output: OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("file\t{}", plan.file.display());
            println!("dialect\t{}", plan.dialect.label());
            println!("strategy\t{}", plan.strategy.label());
            println!("definition_count\t{}", plan.items.len());
            for item in &plan.items {
                println!("old_path\t{}", item.old_path);
                println!("new_path\t{}", item.new_path);
                println!(
                    "span\t{}..{}",
                    item.span.start().get(),
                    item.span.end().get()
                );
                println!("head\t{}", item.head);
                if let Some(name) = &item.name {
                    println!("name\t{name}");
                }
                println!("category\t{}", item.category.label());
            }
            println!("changed\t{}", plan.changed);
            println!("written\t{}", plan.written);
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "command": "sort-definitions",
                "file": plan.file,
                "dialect": plan.dialect.label(),
                "strategy": plan.strategy.label(),
                "definition_count": plan.items.len(),
                "definitions": plan.items.iter().map(|item| {
                    json!({
                        "old_path": item.old_path.to_string(),
                        "new_path": item.new_path.to_string(),
                        "span": {
                            "start": item.span.start().get(),
                            "end": item.span.end().get(),
                        },
                        "head": item.head,
                        "name": item.name,
                        "category": item.category.label(),
                        "source_index": item.source_index,
                        "target_index": item.target_index,
                    })
                }).collect::<Vec<_>>(),
                "rewritten": plan.rewritten,
                "changed": plan.changed,
                "written": plan.written,
            }))?
        ),
    }
    Ok(())
}

pub(super) fn print_move_definition_plan(
    plan: &MoveDefinitionPlan,
    output: OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("from_file\t{}", plan.from_file.display());
            println!("to_file\t{}", plan.to_file.display());
            println!("from_dialect\t{}", plan.from_dialect.label());
            println!("to_dialect\t{}", plan.to_dialect.label());
            println!("path\t{}", plan.path);
            println!(
                "span\t{}..{}",
                plan.span.start().get(),
                plan.span.end().get()
            );
            println!("head\t{}", plan.definition.head);
            if let Some(name) = &plan.definition.name {
                println!("name\t{name}");
            }
            println!("category\t{}", plan.definition.category.label());
            println!("to_file_existed\t{}", plan.to_file_existed);
            println!("changed\t{}", plan.changed);
            println!("written\t{}", plan.written);
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "from_file": plan.from_file,
                "to_file": plan.to_file,
                "from_dialect": plan.from_dialect.label(),
                "to_dialect": plan.to_dialect.label(),
                "path": plan.path.to_string(),
                "span": {
                    "start": plan.span.start().get(),
                    "end": plan.span.end().get(),
                },
                "definition": {
                    "path": plan.definition.path,
                    "span": {
                        "start": plan.definition.span.start().get(),
                        "end": plan.definition.span.end().get(),
                    },
                    "head": plan.definition.head,
                    "name": plan.definition.name,
                    "category": plan.definition.category.label(),
                    "parameter_count": plan.definition.parameter_count,
                    "body_form_count": plan.definition.body_form_count,
                    "package": plan.definition.package,
                    "text": plan.definition_text,
                },
                "from_rewritten": plan.from_rewritten,
                "to_rewritten": plan.to_rewritten,
                "to_file_existed": plan.to_file_existed,
                "changed": plan.changed,
                "written": plan.written,
            }))?
        ),
    }
    Ok(())
}

pub(super) fn print_split_file_plan(plan: &SplitFilePlan, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("from_file\t{}", plan.from_file.display());
            println!("to_file\t{}", plan.to_file.display());
            println!("from_dialect\t{}", plan.from_dialect.label());
            println!("to_dialect\t{}", plan.to_dialect.label());
            println!("definition_count\t{}", plan.items.len());
            for item in &plan.items {
                println!("path\t{}", item.path);
                println!(
                    "span\t{}..{}",
                    item.span.start().get(),
                    item.span.end().get()
                );
                println!("head\t{}", item.definition.head);
                if let Some(name) = &item.definition.name {
                    println!("name\t{name}");
                }
                println!("category\t{}", item.definition.category.label());
            }
            println!("to_file_existed\t{}", plan.to_file_existed);
            println!("to_parent_existed\t{}", plan.to_parent_existed);
            println!("changed\t{}", plan.changed);
            println!("written\t{}", plan.written);
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "command": "split-file",
                "from_file": plan.from_file,
                "to_file": plan.to_file,
                "from_dialect": plan.from_dialect.label(),
                "to_dialect": plan.to_dialect.label(),
                "definition_count": plan.items.len(),
                "definitions": plan.items.iter().map(|item| {
                    json!({
                        "path": item.path.to_string(),
                        "span": {
                            "start": item.span.start().get(),
                            "end": item.span.end().get(),
                        },
                        "removal_span": {
                            "start": item.removal_span.start().get(),
                            "end": item.removal_span.end().get(),
                        },
                        "head": item.definition.head,
                        "name": item.definition.name,
                        "category": item.definition.category.label(),
                        "parameter_count": item.definition.parameter_count,
                        "body_form_count": item.definition.body_form_count,
                        "package": item.definition.package,
                        "text": item.definition_text,
                    })
                }).collect::<Vec<_>>(),
                "from_rewritten": plan.from_rewritten,
                "to_rewritten": plan.to_rewritten,
                "to_file_existed": plan.to_file_existed,
                "to_parent_existed": plan.to_parent_existed,
                "changed": plan.changed,
                "written": plan.written,
            }))?
        ),
    }
    Ok(())
}

pub(super) fn print_move_form_plan(plan: &MoveFormPlan, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("from_file\t{}", plan.from_file.display());
            println!("to_file\t{}", plan.to_file.display());
            println!("from_dialect\t{}", plan.from_dialect.label());
            println!("to_dialect\t{}", plan.to_dialect.label());
            println!("path\t{}", plan.path);
            println!(
                "span\t{}..{}",
                plan.span.start().get(),
                plan.span.end().get()
            );
            if let Some(head) = &plan.head {
                println!("head\t{head}");
            }
            println!("insert\t{}", plan.insert.label());
            if let Some(anchor_path) = &plan.anchor_path {
                println!("anchor_path\t{anchor_path}");
            }
            if let Some(anchor_span) = plan.anchor_span {
                println!(
                    "anchor_span\t{}..{}",
                    anchor_span.start().get(),
                    anchor_span.end().get()
                );
            }
            println!("to_file_existed\t{}", plan.to_file_existed);
            println!("changed\t{}", plan.changed);
            println!("written\t{}", plan.written);
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "from_file": plan.from_file,
                "to_file": plan.to_file,
                "from_dialect": plan.from_dialect.label(),
                "to_dialect": plan.to_dialect.label(),
                "path": plan.path.to_string(),
                "span": {
                    "start": plan.span.start().get(),
                    "end": plan.span.end().get(),
                },
                "head": plan.head,
                "text": plan.form_text,
                "insert": plan.insert.label(),
                "anchor_path": plan.anchor_path.as_ref().map(ToString::to_string),
                "anchor_span": plan.anchor_span.map(|span| {
                    json!({
                        "start": span.start().get(),
                        "end": span.end().get(),
                    })
                }),
                "from_rewritten": plan.from_rewritten,
                "to_rewritten": plan.to_rewritten,
                "to_file_existed": plan.to_file_existed,
                "changed": plan.changed,
                "written": plan.written,
            }))?
        ),
    }
    Ok(())
}
