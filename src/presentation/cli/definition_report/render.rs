use super::super::*;
use crate::application::definition_report::{
    DefinitionReportFile, UnusedDefinitionFile, UnusedDefinitionPolicy,
};

pub(in crate::presentation::cli) fn print_definition_report(
    reports: &[DefinitionReportFile],
    output: OutputFormat,
) -> Result<()> {
    let definition_count = reports
        .iter()
        .map(|report| report.definitions.len())
        .sum::<usize>();
    let mut by_category = BTreeMap::<DefinitionCategory, usize>::new();
    for definition in reports.iter().flat_map(|report| &report.definitions) {
        *by_category.entry(definition.category).or_default() += 1;
    }

    match output {
        OutputFormat::Text => {
            println!("files\t{}", reports.len());
            println!("definition_count\t{definition_count}");
            for (category, count) in &by_category {
                println!("category\t{}\t{count}", category.label());
            }
            for report in reports {
                println!(
                    "{}\t{}\tdefinitions={}\tpackage={}",
                    report.path.display(),
                    report.dialect.label(),
                    report.definitions.len(),
                    report.package.as_deref().unwrap_or("")
                );
                for definition in &report.definitions {
                    println!(
                        "\t{}\t{}\t{}\t{}..{}\tparams={}\tbody={}\tpackage={}",
                        definition.category.label(),
                        definition.head,
                        definition.name.as_deref().unwrap_or(""),
                        definition.span.start().get(),
                        definition.span.end().get(),
                        definition
                            .parameter_count
                            .map(|count| count.to_string())
                            .unwrap_or_default(),
                        definition
                            .body_form_count
                            .map(|count| count.to_string())
                            .unwrap_or_default(),
                        definition.package.as_deref().unwrap_or("")
                    );
                }
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "file_count": reports.len(),
                "definition_count": definition_count,
                "by_category": by_category
                    .iter()
                    .map(|(category, count)| json!({
                        "category": category.label(),
                        "count": count,
                    }))
                    .collect::<Vec<_>>(),
                "files": reports
                    .iter()
                    .map(|report| json!({
                        "path": report.path.display().to_string(),
                        "dialect": report.dialect.label(),
                        "package": report.package.as_deref(),
                        "definition_count": report.definitions.len(),
                        "definitions": report
                            .definitions
                            .iter()
                            .map(|definition| json!({
                                "path": definition.path.as_str(),
                                "span": {
                                    "start": definition.span.start().get(),
                                    "end": definition.span.end().get(),
                                },
                                "head": definition.head.as_str(),
                                "name": definition.name.as_deref(),
                                "category": definition.category.label(),
                                "parameter_count": definition.parameter_count,
                                "body_form_count": definition.body_form_count,
                                "package": definition.package.as_deref(),
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }

    Ok(())
}

pub(in crate::presentation::cli) fn print_unused_definition_report(
    reports: &[UnusedDefinitionFile],
    policy: &UnusedDefinitionPolicy,
    output: OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("files\t{}", reports.len());
            println!("definition_count\t{}", policy.definition_count);
            println!("candidate_count\t{}", policy.candidate_count);
            println!("policy_passed\t{}", policy.passed);
            for violation in &policy.violations {
                println!("policy_violation\t{violation}");
            }
            for report in reports {
                let report_candidate_count = report
                    .definitions
                    .iter()
                    .filter(|item| item.references.is_empty())
                    .count();
                println!(
                    "{}\t{}\tdefinitions={}\tcandidates={}\tpackage={}",
                    report.path.display(),
                    report.dialect.label(),
                    report.definitions.len(),
                    report_candidate_count,
                    report.package.as_deref().unwrap_or("")
                );
                for item in &report.definitions {
                    let definition = &item.definition;
                    println!(
                        "\t{}\t{}\t{}\t{}..{}\treferences={}\tunused={}",
                        definition.category.label(),
                        definition.head,
                        definition.name.as_deref().unwrap_or(""),
                        definition.span.start().get(),
                        definition.span.end().get(),
                        item.references.len(),
                        item.references.is_empty()
                    );
                }
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "file_count": reports.len(),
                "definition_count": policy.definition_count,
                "candidate_count": policy.candidate_count,
                "policy": {
                    "fail_on_unused": policy.fail_on_unused,
                    "require_unused_definitions": policy.require_unused_definitions,
                    "passed": policy.passed,
                    "violations": &policy.violations,
                },
                "candidates": reports
                    .iter()
                    .flat_map(|report| {
                        report
                            .definitions
                            .iter()
                            .filter(|item| item.references.is_empty())
                            .map(|item| {
                                let definition = &item.definition;
                                json!({
                                    "file": report.path.display().to_string(),
                                    "dialect": report.dialect.label(),
                                    "package": report.package.as_deref(),
                                    "path": definition.path.as_str(),
                                    "span": {
                                        "start": definition.span.start().get(),
                                        "end": definition.span.end().get(),
                                    },
                                    "head": definition.head.as_str(),
                                    "name": definition.name.as_deref(),
                                    "category": definition.category.label(),
                                })
                            })
                    })
                    .collect::<Vec<_>>(),
                "files": reports
                    .iter()
                    .map(|report| json!({
                        "path": report.path.display().to_string(),
                        "dialect": report.dialect.label(),
                        "package": report.package.as_deref(),
                        "definition_count": report.definitions.len(),
                        "candidate_count": report
                            .definitions
                            .iter()
                            .filter(|item| item.references.is_empty())
                            .count(),
                        "definitions": report
                            .definitions
                            .iter()
                            .map(|item| {
                                let definition = &item.definition;
                                json!({
                                    "path": definition.path.as_str(),
                                    "span": {
                                        "start": definition.span.start().get(),
                                        "end": definition.span.end().get(),
                                    },
                                    "head": definition.head.as_str(),
                                    "name": definition.name.as_deref(),
                                    "category": definition.category.label(),
                                    "parameter_count": definition.parameter_count,
                                    "body_form_count": definition.body_form_count,
                                    "package": definition.package.as_deref(),
                                    "reference_count": item.references.len(),
                                    "unused": item.references.is_empty(),
                                    "references": item
                                        .references
                                        .iter()
                                        .map(|reference| json!({
                                            "file": reports[reference.file_index].path.display().to_string(),
                                            "path": reference.path.as_str(),
                                            "span": {
                                                "start": reference.span.start().get(),
                                                "end": reference.span.end().get(),
                                            },
                                        }))
                                        .collect::<Vec<_>>(),
                                })
                            })
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }

    Ok(())
}
