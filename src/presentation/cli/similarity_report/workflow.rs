use std::path::PathBuf;
use std::thread;

use anyhow::{Result, bail};

use crate::application::usecase::similarity_report::{
    SimilarityCandidate, SimilarityReportOptions, build_similarity_pairs,
    collect_similarity_candidates,
};
use crate::domain::sexpr::SyntaxTree;
use crate::infrastructure::workspace::{WorkspaceDiscoveryOptions, discover_workspace_files};

use super::super::read_input_and_dialect;
use super::args::SimilarityReportArgs;
use super::render::print_similarity_report;
use super::types::{ErrorPolicy, FileProcessingError};

pub fn similarity_report(args: SimilarityReportArgs) -> Result<()> {
    ensure_options(
        args.threshold,
        args.min_node_count,
        args.min_line_span,
        args.max_candidates,
        args.max_comparisons,
        args.max_results,
    )?;

    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown || args.dialect.is_some(),
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        max_depth: args.max_depth,
        exclude: args.exclude.clone(),
    })?;

    let options = SimilarityReportOptions {
        threshold: args.threshold,
        min_node_count: args.min_node_count,
        min_line_span: args.min_line_span,
        comparison_scope: args.comparison_scope,
        form_scope: args.form_scope,
        overlap_policy: args.overlap_policy,
        max_candidates: args.max_candidates,
        max_comparisons: args.max_comparisons,
        max_results: args.max_results,
    };
    let output = process_workspace_files(
        discovery.files.clone(),
        args.dialect,
        &options,
        args.error_policy,
    );

    if args.error_policy == ErrorPolicy::Fail
        && let Some(error) = output.errors.first()
    {
        return Err(anyhow::anyhow!(
            "failed to {} {}: {}",
            error.stage,
            error.path.display(),
            error.source
        ));
    }

    let errors: Vec<FileProcessingError> = output
        .errors
        .iter()
        .map(|error| FileProcessingError {
            path: error.path.clone(),
            stage: error.stage,
            message: error.source.root_cause().to_string(),
        })
        .collect();

    let mut report = build_similarity_pairs(output.candidates, &options);
    report.summary.candidate_limit_reached = output.omitted_candidates > 0;
    report.summary.omitted_candidates = output.omitted_candidates;
    print_similarity_report(&report, &discovery, &errors, &args)?;

    if args.fail_on_duplicates && report.summary.matched_pairs > 0 {
        return Err(crate::presentation::cli::gate::gate_failure(format!(
            "similarity-report policy failed: {} duplicate pair(s) found",
            report.summary.matched_pairs
        )));
    }
    if args.fail_on_duplicates && report.summary.comparison_limit_reached {
        bail!(
            "similarity-report policy indeterminate: comparison limit reached with {} pair(s) unprocessed",
            report.summary.unprocessed_pairs
        );
    }
    if args.fail_on_duplicates && report.summary.candidate_limit_reached {
        bail!(
            "similarity-report policy indeterminate: candidate limit reached with {} candidate(s) omitted",
            report.summary.omitted_candidates
        );
    }
    if args.fail_on_duplicates && !errors.is_empty() {
        bail!(
            "similarity-report policy indeterminate: {} file(s) skipped due to processing errors",
            errors.len()
        );
    }

    Ok(())
}

#[derive(Debug)]
struct ProcessingError {
    path: PathBuf,
    stage: &'static str,
    source: anyhow::Error,
}

#[derive(Default)]
struct WorkspaceProcessingOutput {
    candidates: Vec<SimilarityCandidate>,
    errors: Vec<ProcessingError>,
    omitted_candidates: usize,
}

fn process_workspace_files(
    files: Vec<PathBuf>,
    dialect: Option<super::super::DialectArg>,
    options: &SimilarityReportOptions,
    error_policy: ErrorPolicy,
) -> WorkspaceProcessingOutput {
    if files.is_empty() {
        return WorkspaceProcessingOutput::default();
    }

    let worker_count = thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1)
        .max(1);
    let chunk_size = files.len().max(1).div_ceil(worker_count);
    let mut handles = Vec::new();

    for chunk in files.chunks(chunk_size) {
        let files = chunk.to_owned();
        let options = options.clone();
        handles.push(thread::spawn(move || {
            process_file_chunk(files, dialect, &options, error_policy)
        }));
    }

    let mut merged = WorkspaceProcessingOutput::default();
    for handle in handles {
        if let Ok(output) = handle.join() {
            merged.candidates.extend(output.candidates);
            merged.errors.extend(output.errors);
            merged.omitted_candidates = merged
                .omitted_candidates
                .saturating_add(output.omitted_candidates);
        }
    }

    merged
}

fn process_file_chunk(
    files: Vec<PathBuf>,
    dialect: Option<super::super::DialectArg>,
    options: &SimilarityReportOptions,
    error_policy: ErrorPolicy,
) -> WorkspaceProcessingOutput {
    let mut output = WorkspaceProcessingOutput::default();
    for file in files {
        match process_file(&file, dialect, options) {
            Ok(file_output) => {
                output.candidates.extend(file_output.candidates);
                output.omitted_candidates = output
                    .omitted_candidates
                    .saturating_add(file_output.omitted_candidates);
            }
            Err(error) => {
                if error_policy == ErrorPolicy::Fail {
                    output.errors.push(error);
                    break;
                }
                output.errors.push(error);
            }
        }
    }
    output
}

struct FileProcessingOutput {
    candidates: Vec<SimilarityCandidate>,
    omitted_candidates: usize,
}

fn process_file(
    file: &std::path::Path,
    dialect: Option<super::super::DialectArg>,
    options: &SimilarityReportOptions,
) -> std::result::Result<FileProcessingOutput, ProcessingError> {
    let (input, dialect) =
        read_input_and_dialect(Some(file.to_path_buf()), dialect).map_err(|source| {
            ProcessingError {
                path: file.to_path_buf(),
                stage: "read",
                source,
            }
        })?;
    let tree = SyntaxTree::parse(&input.text).map_err(|source| ProcessingError {
        path: file.to_path_buf(),
        stage: "parse",
        source: source.into(),
    })?;
    let mut candidates = Vec::new();
    let omitted_candidates =
        collect_similarity_candidates(&tree, &input.text, file, dialect, options, &mut candidates)
            .map_err(|source| ProcessingError {
                path: file.to_path_buf(),
                stage: "collect",
                source,
            })?;

    Ok(FileProcessingOutput {
        candidates,
        omitted_candidates,
    })
}

fn ensure_options(
    threshold: f64,
    min_node_count: usize,
    min_line_span: usize,
    max_candidates: Option<usize>,
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
    if max_candidates == Some(0) {
        bail!("--max-candidates must be at least 1");
    }
    Ok(())
}
