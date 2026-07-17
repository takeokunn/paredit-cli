use std::path::PathBuf;
use std::thread;

use anyhow::{Result, bail};

use crate::application::usecase::similarity_report::{
    SimilarityCandidate, SimilarityReportOptions, build_similarity_pairs_with_omissions,
    collect_similarity_candidates,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::SyntaxTree;
use crate::infrastructure::workspace::{
    WorkspaceDiscovery, WorkspaceDiscoveryOptions, discover_workspace_files,
};

use super::args::SimilarityReportArgs;
use super::render::print_similarity_report;
use super::types::{ErrorPolicy, FileProcessingError};

pub fn similarity_report(args: SimilarityReportArgs) -> Result<()> {
    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown || args.dialect.is_some(),
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        max_depth: args.max_depth,
        exclude: args.exclude.clone(),
    })?;

    let options = SimilarityReportOptions::new(
        args.threshold,
        args.min_node_count,
        args.min_line_span,
        args.comparison_scope,
        args.form_scope,
        args.overlap_policy,
        args.max_candidates,
        args.max_comparisons,
        args.max_results,
    )?;
    let output = process_workspace_files(&discovery, args.dialect, &options, args.error_policy)?;

    if args.error_policy == ErrorPolicy::Fail {
        if let Some(error) = output.errors.first() {
            return Err(anyhow::anyhow!(
                "failed to {} {}: {}",
                error.stage,
                error.path.display(),
                error.source
            ));
        }
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

    let report = build_similarity_pairs_with_omissions(
        output.candidates,
        output.omitted_candidates,
        &options,
    )?;
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
    discovery: &WorkspaceDiscovery,
    dialect: Option<super::super::DialectArg>,
    options: &SimilarityReportOptions,
    error_policy: ErrorPolicy,
) -> Result<WorkspaceProcessingOutput> {
    if discovery.files().is_empty() {
        return Ok(WorkspaceProcessingOutput::default());
    }

    let worker_count = thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1)
        .max(1);
    let chunk_size = discovery.files().len().max(1).div_ceil(worker_count);
    let mut handles = Vec::new();

    for chunk in discovery.files().chunks(chunk_size) {
        let files = chunk.to_owned();
        let options = options.clone();
        let discovery = discovery.clone();
        handles.push(thread::spawn(move || {
            process_file_chunk(files, &discovery, dialect, &options, error_policy)
        }));
    }

    let mut merged = WorkspaceProcessingOutput::default();
    for handle in handles {
        let output = handle
            .join()
            .map_err(|_| anyhow::anyhow!("similarity-report worker thread panicked"))?;
        merged.candidates.extend(output.candidates);
        merged.errors.extend(output.errors);
        merged.omitted_candidates = merged
            .omitted_candidates
            .saturating_add(output.omitted_candidates);
    }

    Ok(merged)
}

fn process_file_chunk(
    files: Vec<PathBuf>,
    discovery: &WorkspaceDiscovery,
    dialect: Option<super::super::DialectArg>,
    options: &SimilarityReportOptions,
    error_policy: ErrorPolicy,
) -> WorkspaceProcessingOutput {
    let mut output = WorkspaceProcessingOutput::default();
    for file in files {
        match process_file(&file, discovery, dialect, options) {
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
    discovery: &WorkspaceDiscovery,
    dialect: Option<super::super::DialectArg>,
    options: &SimilarityReportOptions,
) -> std::result::Result<FileProcessingOutput, ProcessingError> {
    let bytes = discovery
        .read_file(file)
        .map_err(|source| ProcessingError {
            path: file.to_path_buf(),
            stage: "read",
            source,
        })?;
    let text = String::from_utf8(bytes).map_err(|source| ProcessingError {
        path: file.to_path_buf(),
        stage: "read",
        source: source.into(),
    })?;
    let dialect = Dialect::detect(Some(file), dialect.map(Into::into));
    let tree = SyntaxTree::parse(&text).map_err(|source| ProcessingError {
        path: file.to_path_buf(),
        stage: "parse",
        source: source.into(),
    })?;
    let mut candidates = Vec::new();
    let omitted_candidates =
        collect_similarity_candidates(&tree, &text, file, dialect, options, &mut candidates)
            .map_err(|source| ProcessingError {
                path: file.to_path_buf(),
                stage: "collect",
                source: source.into(),
            })?;

    Ok(FileProcessingOutput {
        candidates,
        omitted_candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::usecase::similarity_report::{
        SimilarityComparisonScope, SimilarityFormScope, SimilarityOverlapPolicy,
    };

    #[test]
    fn empty_workspace_returns_empty_processing_output() {
        let options = SimilarityReportOptions::new(
            0.87,
            4,
            1,
            SimilarityComparisonScope::All,
            SimilarityFormScope::All,
            SimilarityOverlapPolicy::Maximal,
            None,
            None,
            None,
        )
        .expect("test options are valid");

        let output = process_workspace_files(
            &WorkspaceDiscovery::default(),
            None,
            &options,
            ErrorPolicy::Fail,
        )
        .expect("empty workspace should not fail");

        assert!(output.candidates.is_empty());
        assert!(output.errors.is_empty());
        assert_eq!(output.omitted_candidates, 0);
    }
}
