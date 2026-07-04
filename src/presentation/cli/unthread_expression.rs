use super::*;
use crate::application::usecase::unthread_expression::{
    UnthreadExpressionPlan, UnthreadExpressionRequest, UnthreadStyle as ApplicationUnthreadStyle,
    plan_unthread_expression,
};

#[derive(Debug, Args)]
pub(super) struct UnthreadExpressionArgs {
    /// Input file. Required when --write is used; reads stdin otherwise.
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Override extension-based dialect detection.
    #[arg(long)]
    dialect: Option<DialectArg>,
    /// Select by child index path, for example 0.2.1.
    #[arg(long, conflicts_with = "at")]
    path: Option<Path>,
    /// Select the smallest expression containing byte offset.
    #[arg(long, conflicts_with = "path")]
    at: Option<usize>,
    /// Threading style. Required for custom operators other than -> and ->>.
    #[arg(long, value_enum)]
    style: Option<ThreadStyleArg>,
    /// Expected threading operator. Defaults to the selected form head.
    #[arg(long)]
    operator: Option<SymbolName>,
    /// Rewrite the input file in place. Without this flag, only prints a plan.
    #[arg(long)]
    write: bool,
    /// Output format for agent consumption.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
}

impl ThreadStyleArg {
    fn application_unthread_style(self) -> ApplicationUnthreadStyle {
        match self {
            Self::First => ApplicationUnthreadStyle::First,
            Self::Last => ApplicationUnthreadStyle::Last,
        }
    }
}

pub(super) fn unthread_expression(args: UnthreadExpressionArgs) -> Result<()> {
    let input = read_input(args.file.clone())?;
    let dialect = detect_dialect(&input, args.dialect);
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    let selected = selection.view();
    let path = args.path.clone();
    let style = args.style.map(ThreadStyleArg::application_unthread_style);
    let plan = plan_unthread_expression(UnthreadExpressionRequest {
        input: &input.text,
        dialect,
        path,
        target: selected,
        style,
        operator: args.operator,
    })?;
    let mut written = false;

    if args.write {
        let file = input.file.as_ref().context("--write requires --file")?;
        if plan.changed {
            fs::write(file, &plan.rewritten)
                .with_context(|| format!("write {}", file.display()))?;
        }
        written = true;
    }

    print_unthread_expression_plan(&plan, written, args.output)
}

fn print_unthread_expression_plan(
    plan: &UnthreadExpressionPlan,
    written: bool,
    output: OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("dialect\t{}", plan.dialect.label());
            if let Some(path) = &plan.path {
                println!("path\t{path}");
            }
            println!("style\t{}", plan.style.label());
            println!("operator\t{}", plan.operator);
            println!(
                "span\t{}..{}",
                plan.span.start().get(),
                plan.span.end().get()
            );
            println!("base\t{}", plan.base);
            for step in &plan.steps {
                println!(
                    "step\t{}\targs={}\tinsertion_index={}\tspan={}..{}\t{}",
                    step.head,
                    step.argument_count,
                    step.insertion_index,
                    step.span.start().get(),
                    step.span.end().get(),
                    step.form
                );
            }
            println!("replacement\t{}", plan.replacement);
            println!("changed\t{}", plan.changed);
            println!("written\t{written}");
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "dialect": plan.dialect.label(),
                "path": plan.path.as_ref().map(ToString::to_string),
                "style": plan.style.label(),
                "operator": plan.operator.as_str(),
                "span": {
                    "start": plan.span.start().get(),
                    "end": plan.span.end().get(),
                },
                "base": &plan.base,
                "steps": plan.steps.iter().map(|step| json!({
                    "head": &step.head,
                    "argument_count": step.argument_count,
                    "insertion_index": step.insertion_index,
                    "span": {
                        "start": step.span.start().get(),
                        "end": step.span.end().get(),
                    },
                    "form": &step.form,
                })).collect::<Vec<_>>(),
                "replacement": &plan.replacement,
                "changed": plan.changed,
                "written": written,
                "rewritten": &plan.rewritten,
            }))?
        ),
    }
    Ok(())
}
