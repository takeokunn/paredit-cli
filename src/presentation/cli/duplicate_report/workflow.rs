use super::super::*;
use super::args::{DuplicateReportArgs, ReplacementPlanArgs};
use super::render::{print_duplicate_report, print_replacement_plan};
use crate::application::usecase::duplicate_report::{
    DuplicateCandidateGroups, build_duplicate_shape_reports, collect_duplicate_candidates,
    collect_replacement_plan_batches,
};

pub(in crate::presentation::cli) fn duplicate_report(args: DuplicateReportArgs) -> Result<()> {
    ensure_thresholds(args.min_group_size, args.min_node_count)?;

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

pub(in crate::presentation::cli) fn replacement_plan(args: ReplacementPlanArgs) -> Result<()> {
    ensure_thresholds(args.min_group_size, args.min_node_count)?;

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

    let mut batches = collect_replacement_plan_batches(
        grouped,
        args.min_group_size,
        args.replacement,
        args.keep_first,
    );
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

fn ensure_thresholds(min_group_size: usize, min_node_count: usize) -> Result<()> {
    anyhow::ensure!(min_group_size >= 2, "--min-group-size must be at least 2");
    anyhow::ensure!(min_node_count >= 2, "--min-node-count must be at least 2");
    Ok(())
}
