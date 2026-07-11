use anyhow::{bail, Result};

use crate::application::usecase::similarity_report::{
    build_similarity_pairs, collect_similarity_candidates, SimilarityReportOptions,
};
use crate::domain::sexpr::SyntaxTree;
use crate::infrastructure::workspace::{discover_workspace_files, WorkspaceDiscoveryOptions};

use super::super::{detect_dialect, read_input};
use super::args::SimilarityReportArgs;
use super::render::print_similarity_report;
use super::types::{ErrorPolicy, FileProcessingError};

pub fn similarity_report(args: SimilarityReportArgs) -> Result<()> {
    ensure_options(
        args.threshold,
        args.min_node_count,
        args.min_line_span,
        args.max_comparisons,
        args.max_results,
    )?;

    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown || args.dialect.is_some(),
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        max_depth: args.max_depth,
    })?;

    let mut candidates = Vec::new();
    let mut errors = Vec::new();
    let options = SimilarityReportOptions {
        threshold: args.threshold,
        min_node_count: args.min_node_count,
        min_line_span: args.min_line_span,
        comparison_scope: args.comparison_scope,
        form_scope: args.form_scope,
        overlap_policy: args.overlap_policy,
        max_comparisons: args.max_comparisons,
        max_results: args.max_results,
    };
    for file in &discovery.files {
        if let Err(error) = process_file(file, args.dialect, &options, &mut candidates) {
            if args.error_policy == ErrorPolicy::Fail {
                return Err(error.source.context(format!(
                    "failed to {} {}",
                    error.stage,
                    file.display()
                )));
            }
            errors.push(FileProcessingError {
                path: file.clone(),
                stage: error.stage,
                message: error.source.root_cause().to_string(),
            });
        }
    }

    let report = build_similarity_pairs(candidates, &options);
    print_similarity_report(&report, &discovery, &errors, &args)?;

    if args.fail_on_duplicates && report.summary.matched_pairs > 0 {
        bail!(
            "similarity-report policy failed: {} duplicate pair(s) found",
            report.summary.matched_pairs
        );
    }

    Ok(())
}

struct ProcessingError {
    stage: &'static str,
    source: anyhow::Error,
}

fn process_file(
    file: &std::path::Path,
    dialect: Option<super::super::DialectArg>,
    options: &SimilarityReportOptions,
    candidates: &mut Vec<crate::application::usecase::similarity_report::SimilarityCandidate>,
) -> std::result::Result<(), ProcessingError> {
    let input = read_input(Some(file.to_path_buf())).map_err(|source| ProcessingError {
        stage: "read",
        source,
    })?;
    let dialect = detect_dialect(&input, dialect);
    let tree = SyntaxTree::parse(&input.text).map_err(|source| ProcessingError {
        stage: "parse",
        source: source.into(),
    })?;
    let mut file_candidates = Vec::new();
    collect_similarity_candidates(
        &tree,
        &input.text,
        file,
        dialect,
        options,
        &mut file_candidates,
    )
    .map_err(|source| ProcessingError {
        stage: "collect",
        source,
    })?;
    candidates.extend(file_candidates);
    Ok(())
}

fn ensure_options(
    threshold: f64,
    min_node_count: usize,
    min_line_span: usize,
    max_comparisons: Option<usize>,
    max_results: Option<usize>,
) -> Result<()> {
    if !(0.0..=1.0).contains(&threshold) {
        bail!("--threshold must be between 0.0 and 1.0");
    }
    if min_node_count < 2 {
        bail!("--min-node-count must be at least 2");
    }
    if min_line_span == 0 {
        bail!("--min-line-span must be at least 1");
    }
    if max_results == Some(0) {
        bail!("--max-results must be at least 1");
    }
    if max_comparisons == Some(0) {
        bail!("--max-comparisons must be at least 1");
    }
    Ok(())
}
