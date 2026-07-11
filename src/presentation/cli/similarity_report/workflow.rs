use anyhow::{bail, Context, Result};

use crate::application::usecase::similarity_report::{
    build_similarity_pairs, collect_similarity_candidates,
};
use crate::domain::sexpr::SyntaxTree;
use crate::infrastructure::workspace::{discover_workspace_files, WorkspaceDiscoveryOptions};

use super::super::{detect_dialect, read_input};
use super::args::SimilarityReportArgs;
use super::render::print_similarity_report;

pub fn similarity_report(args: SimilarityReportArgs) -> Result<()> {
    ensure_options(args.threshold, args.min_node_count, args.max_results)?;

    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown || args.dialect.is_some(),
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        max_depth: args.max_depth,
    })?;

    let mut candidates = Vec::new();
    for file in &discovery.files {
        let input = read_input(Some(file.clone()))
            .with_context(|| format!("failed to read {}", file.display()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        collect_similarity_candidates(
            &tree,
            &input.text,
            file,
            dialect,
            args.min_node_count,
            &mut candidates,
        )?;
    }

    let report = build_similarity_pairs(
        candidates,
        args.threshold,
        args.overlap_policy,
        args.max_results,
    );
    print_similarity_report(&report, &discovery, &args)?;

    if args.fail_on_duplicates && report.summary.matched_pairs > 0 {
        bail!(
            "similarity-report policy failed: {} duplicate pair(s) found",
            report.summary.matched_pairs
        );
    }

    Ok(())
}

fn ensure_options(threshold: f64, min_node_count: usize, max_results: Option<usize>) -> Result<()> {
    if !(0.0..=1.0).contains(&threshold) {
        bail!("--threshold must be between 0.0 and 1.0");
    }
    if min_node_count < 2 {
        bail!("--min-node-count must be at least 2");
    }
    if max_results == Some(0) {
        bail!("--max-results must be at least 1");
    }
    Ok(())
}
