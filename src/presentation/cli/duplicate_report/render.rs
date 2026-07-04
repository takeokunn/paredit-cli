use super::super::*;
use crate::application::usecase::duplicate_report::{DuplicateShapeReport, ReplacementPlanBatch};

pub(in crate::presentation::cli) fn print_duplicate_report(
    reports: &[DuplicateShapeReport],
    output: OutputFormat,
) -> Result<()> {
    let form_count = reports
        .iter()
        .map(|report| report.forms.len())
        .sum::<usize>();

    match output {
        OutputFormat::Text => {
            println!("group_count\t{}", reports.len());
            println!("form_count\t{form_count}");
            for report in reports {
                println!("shape\t{}\tcount={}", report.shape, report.count);
                for form in &report.forms {
                    println!(
                        "\t{}\t{}\t{}\t{}..{}\tnodes={}\thead={}",
                        form.path.display(),
                        form.dialect.label(),
                        form.form_path,
                        form.span.start().get(),
                        form.span.end().get(),
                        form.node_count,
                        form.head.as_deref().unwrap_or("")
                    );
                }
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "group_count": reports.len(),
                "form_count": form_count,
                "groups": reports
                    .iter()
                    .map(|report| json!({
                        "shape": report.shape.as_str(),
                        "count": report.count,
                        "forms": report
                            .forms
                            .iter()
                            .map(|form| json!({
                                "path": form.path.display().to_string(),
                                "dialect": form.dialect.label(),
                                "form_path": form.form_path.as_str(),
                                "span": {
                                    "start": form.span.start().get(),
                                    "end": form.span.end().get(),
                                },
                                "node_count": form.node_count,
                                "head": form.head.as_deref(),
                                "text": form.text.as_str(),
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }

    Ok(())
}

pub(in crate::presentation::cli) fn print_replacement_plan(
    batches: &[ReplacementPlanBatch],
    output: OutputFormat,
) -> Result<()> {
    let form_count = batches.iter().map(|batch| batch.forms.len()).sum::<usize>();

    match output {
        OutputFormat::Text => {
            println!("batch_count\t{}", batches.len());
            println!("form_count\t{form_count}");
            for batch in batches {
                let command_args = replace_forms_command_args(batch);
                println!(
                    "batch\t{}\t{}\tcount={}\tshape={}",
                    batch.file.display(),
                    batch.dialect.label(),
                    batch.forms.len(),
                    batch.shape
                );
                println!("command\t{}", format_command(&command_args));
                for form in &batch.forms {
                    println!(
                        "\t{}\t{}..{}\tnodes={}\thead={}",
                        form.form_path,
                        form.span.start().get(),
                        form.span.end().get(),
                        form.node_count,
                        form.head.as_deref().unwrap_or("")
                    );
                }
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "batch_count": batches.len(),
                "form_count": form_count,
                "batches": batches
                    .iter()
                    .map(|batch| {
                        let command_args = replace_forms_command_args(batch);
                        json!({
                            "file": batch.file.display().to_string(),
                            "dialect": batch.dialect.label(),
                            "shape": batch.shape.as_str(),
                            "count": batch.forms.len(),
                            "replacement": batch.replacement.as_str(),
                            "paths": batch
                                .forms
                                .iter()
                                .map(|form| form.form_path.as_str())
                                .collect::<Vec<_>>(),
                            "replace_forms_args": command_args,
                            "command": format_command(&replace_forms_command_args(batch)),
                            "forms": batch
                                .forms
                                .iter()
                                .map(|form| json!({
                                    "form_path": form.form_path.as_str(),
                                    "span": {
                                        "start": form.span.start().get(),
                                        "end": form.span.end().get(),
                                    },
                                    "node_count": form.node_count,
                                    "head": form.head.as_deref(),
                                    "text": form.text.as_str(),
                                }))
                                .collect::<Vec<_>>(),
                        })
                    })
                    .collect::<Vec<_>>(),
            }))?
        ),
    }

    Ok(())
}

fn replace_forms_command_args(batch: &ReplacementPlanBatch) -> Vec<String> {
    let mut args = vec![
        "paredit".to_owned(),
        "replace-forms".to_owned(),
        "--file".to_owned(),
        batch.file.display().to_string(),
    ];

    for form in &batch.forms {
        args.push("--path".to_owned());
        args.push(form.form_path.clone());
    }

    args.extend([
        "--with".to_owned(),
        batch.replacement.clone(),
        "--require-same-shape".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ]);

    args
}

fn format_command(args: &[String]) -> String {
    args.iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(arg: &str) -> String {
    if !arg.is_empty()
        && arg.bytes().all(|byte| {
            matches!(
                byte,
                b'a'..=b'z'
                    | b'A'..=b'Z'
                    | b'0'..=b'9'
                    | b'/'
                    | b'.'
                    | b'_'
                    | b'-'
                    | b':'
                    | b'='
            )
        })
    {
        return arg.to_owned();
    }

    format!("'{}'", arg.replace('\'', "'\\''"))
}
