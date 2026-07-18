use super::*;
use crate::application::usecase::conditional_sugar::{
    ConditionalConversionPlan, ConditionalConversionRequest,
};
use crate::presentation::cli::shared::read_input_dialect_and_tree;

#[derive(Debug, Args)]
pub(super) struct ConditionalConversionArgs {
    #[arg(short, long)]
    file: Option<PathBuf>,
    #[arg(long)]
    dialect: Option<DialectArg>,
    #[arg(long)]
    path: Path,
    #[arg(long)]
    write: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

pub(super) fn run(
    args: ConditionalConversionArgs,
    planner: fn(ConditionalConversionRequest<'_>) -> Result<ConditionalConversionPlan>,
) -> Result<()> {
    if args.write && args.file.is_none() {
        anyhow::bail!("--write requires --file");
    }
    let (input, dialect, _) = read_input_dialect_and_tree(args.file.clone(), args.dialect)?;
    let plan = planner(ConditionalConversionRequest {
        input: &input.text,
        dialect,
        path: args.path,
    })?;
    let written = args.write && plan.changed;
    if written {
        write_file_with_rollback(
            require_output_file(input.file.as_ref())?.clone(),
            plan.rewritten.clone(),
        )?;
    }
    match args.output {
        OutputFormat::Text => {
            println!("dialect\t{}", plan.dialect.label());
            println!("path\t{}", safe_text!(plan.path));
            println!("body_count\t{}", plan.body_count);
            println!("changed\t{}", plan.changed);
            println!("written\t{written}");
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(
                &json!({ "dialect": plan.dialect.label(), "path": plan.path.to_string(), "form_span": { "start": plan.form_span.start().get(), "end": plan.form_span.end().get() }, "body_count": plan.body_count, "changed": plan.changed, "written": written, "rewritten": plan.rewritten })
            )?
        ),
    }
    Ok(())
}
