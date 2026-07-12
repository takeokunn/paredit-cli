use super::super::*;
use super::args::{DuplicateReportArgs, ReplacementPlanArgs};
use super::render::{print_duplicate_report, print_replacement_plan};
use super::workspace::discover_duplicate_report_files;
use crate::application::usecase::duplicate_report::{
    DuplicateCandidateGroups, build_duplicate_shape_reports, collect_duplicate_candidates,
    collect_replacement_plan_batches,
};
use crate::presentation::cli::shared::read_input_dialect_and_tree;

pub(in crate::presentation::cli) fn duplicate_report(args: DuplicateReportArgs) -> Result<()> {
    ensure_thresholds(args.min_group_size, args.min_node_count)?;
    let grouped =
        collect_duplicate_candidate_groups(&args.files, args.dialect, args.min_node_count)?;
    let reports = build_duplicate_shape_reports(grouped, args.min_group_size);

    print_duplicate_report(&reports, args.output)
}

pub(in crate::presentation::cli) fn replacement_plan(args: ReplacementPlanArgs) -> Result<()> {
    ensure_thresholds(args.min_group_size, args.min_node_count)?;
    let grouped =
        collect_duplicate_candidate_groups(&args.files, args.dialect, args.min_node_count)?;
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

fn collect_duplicate_candidate_groups(
    roots: &[std::path::PathBuf],
    dialect: Option<super::super::DialectArg>,
    min_node_count: usize,
) -> Result<DuplicateCandidateGroups> {
    let mut grouped = DuplicateCandidateGroups::new();

    for file in discover_duplicate_report_files(roots)? {
        let (input, dialect, tree) = read_input_dialect_and_tree(Some(file.clone()), dialect)?;
        collect_duplicate_candidates(
            &tree,
            &input.text,
            &file,
            dialect,
            min_node_count,
            &mut grouped,
        )?;
    }

    Ok(grouped)
}

fn ensure_thresholds(min_group_size: usize, min_node_count: usize) -> Result<()> {
    anyhow::ensure!(min_group_size >= 2, "--min-group-size must be at least 2");
    anyhow::ensure!(min_node_count >= 2, "--min-node-count must be at least 2");
    Ok(())
}
