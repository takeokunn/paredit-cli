use super::*;
use crate::application::duplicate_report::{
    DuplicateCandidateGroups, DuplicateShapeReport, ReplacementPlanBatch,
    build_duplicate_shape_reports, collect_duplicate_candidates, collect_replacement_plan_batches,
};

#[derive(Debug, Args)]
pub(super) struct DuplicateReportArgs {
    /// Files to scan.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Minimum number of matching forms required for a reported group.
    #[arg(long, default_value_t = 2)]
    min_group_size: usize,
    /// Minimum expression node count for a candidate form.
    #[arg(long, default_value_t = 4)]
    min_node_count: usize,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

#[derive(Debug, Args)]
pub(super) struct ReplacementPlanArgs {
    /// Files to scan.
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Override extension-based dialect detection for every file.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Minimum number of matching forms required in one file for a batch.
    #[arg(long, default_value_t = 2)]
    min_group_size: usize,
    /// Minimum expression node count for a candidate form.
    #[arg(long, default_value_t = 4)]
    min_node_count: usize,
    /// Placeholder replacement form for generated replace-forms commands.
    #[arg(long, default_value = "(TODO-refactor)")]
    replacement: String,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

pub(super) fn duplicate_report(args: DuplicateReportArgs) -> Result<()> {
    anyhow::ensure!(
        args.min_group_size >= 2,
        "--min-group-size must be at least 2"
    );
    anyhow::ensure!(
        args.min_node_count >= 2,
        "--min-node-count must be at least 2"
    );

    let mut grouped = DuplicateCandidateGroups::new();

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        collect_duplicate_candidates(
            &tree,
            &input.text,
            file,
            dialect,
            args.min_node_count,
            &mut grouped,
        )?;
    }

    let reports = build_duplicate_shape_reports(grouped, args.min_group_size);

    print_duplicate_report(&reports, args.output)
}

pub(super) fn replacement_plan(args: ReplacementPlanArgs) -> Result<()> {
    anyhow::ensure!(
        args.min_group_size >= 2,
        "--min-group-size must be at least 2"
    );
    anyhow::ensure!(
        args.min_node_count >= 2,
        "--min-node-count must be at least 2"
    );

    let mut grouped = DuplicateCandidateGroups::new();

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        collect_duplicate_candidates(
            &tree,
            &input.text,
            file,
            dialect,
            args.min_node_count,
            &mut grouped,
        )?;
    }

    let mut batches =
        collect_replacement_plan_batches(grouped, args.min_group_size, args.replacement);
    batches.sort_by(|left, right| {
        right
            .forms
            .len()
            .cmp(&left.forms.len())
            .then_with(|| left.file.cmp(&right.file))
            .then_with(|| left.shape.cmp(&right.shape))
    });

    print_replacement_plan(&batches, args.output)
}

fn print_duplicate_report(reports: &[DuplicateShapeReport], output: OutputFormat) -> Result<()> {
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

fn print_replacement_plan(batches: &[ReplacementPlanBatch], output: OutputFormat) -> Result<()> {
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
