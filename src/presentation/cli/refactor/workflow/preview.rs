use super::super::super::*;
use super::super::args::*;
use super::super::render::print_refactor_preview;
use super::super::types::plan::WorkspaceRefactorPlanDiscovery;
use super::super::types::preview::{RefactorPreview, RefactorPreviewFile};

pub(in crate::presentation::cli) fn refactor_preview(args: RefactorPreviewArgs) -> Result<()> {
    emit_refactor_preview(
        &args.files,
        args.dialect,
        &args.from,
        &args.to,
        args.mode,
        args.max_preview_bytes,
        args.write,
        RefactorPreviewPolicyOptions {
            fail_on_no_change: args.fail_on_no_change,
            fail_on_parse_error: args.fail_on_parse_error,
            fail_on_target_conflict: args.fail_on_target_conflict,
            require_changed_files: args.require_changed_files,
            require_definitions: args.require_definitions,
            require_edits: args.require_edits,
        },
        None,
        args.output,
        "refactor-preview",
    )
}

pub(in crate::presentation::cli) fn workspace_refactor_preview(
    args: WorkspaceRefactorPreviewArgs,
) -> Result<()> {
    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: args.roots.clone(),
        include_unknown: args.include_unknown,
        include_hidden: args.include_hidden,
        include_generated: args.include_generated,
        max_depth: args.max_depth,
    })?;
    let workspace = WorkspaceRefactorPlanDiscovery {
        roots: args.roots,
        discovered_file_count: discovery.files.len(),
        skipped_unknown_count: discovery.skipped_unknown_count,
        skipped_hidden_count: discovery.skipped_hidden_count,
        skipped_generated_count: discovery.skipped_generated_count,
        skipped_symlink_count: discovery.skipped_symlink_count,
    };

    emit_refactor_preview(
        &discovery.files,
        None,
        &args.from,
        &args.to,
        args.mode,
        args.max_preview_bytes,
        args.write,
        RefactorPreviewPolicyOptions {
            fail_on_no_change: args.fail_on_no_change,
            fail_on_parse_error: args.fail_on_parse_error,
            fail_on_target_conflict: args.fail_on_target_conflict,
            require_changed_files: args.require_changed_files,
            require_definitions: args.require_definitions,
            require_edits: args.require_edits,
        },
        Some(workspace),
        args.output,
        "workspace-refactor-preview",
    )
}

pub(super) struct BuildRefactorPreviewRequest<'a> {
    pub(super) paths: &'a [PathBuf],
    pub(super) dialect: Option<DialectArg>,
    pub(super) from: &'a SymbolName,
    pub(super) to: &'a SymbolName,
    pub(super) mode: RefactorPreviewMode,
    pub(super) max_preview_bytes: usize,
    pub(super) write: bool,
    pub(super) policy_options: RefactorPreviewPolicyOptions,
    pub(super) workspace: Option<WorkspaceRefactorPlanDiscovery>,
}

fn emit_refactor_preview(
    paths: &[PathBuf],
    dialect: Option<DialectArg>,
    from: &SymbolName,
    to: &SymbolName,
    mode: RefactorPreviewMode,
    max_preview_bytes: usize,
    write: bool,
    policy_options: RefactorPreviewPolicyOptions,
    workspace: Option<WorkspaceRefactorPlanDiscovery>,
    output: OutputFormat,
    failure_label: &'static str,
) -> Result<()> {
    let mut preview = build_refactor_preview(BuildRefactorPreviewRequest {
        paths,
        dialect,
        from,
        to,
        mode,
        max_preview_bytes,
        write,
        policy_options,
        workspace,
    })?;
    let policy_passed = preview.policy.passed;
    let policy_message = preview.policy.violations.join("; ");
    let write_parse_refused = write && !preview.summary.all_outputs_parse;

    if policy_passed && !write_parse_refused {
        write_refactor_preview(&mut preview)?;
    }

    print_refactor_preview(&preview, output)?;
    finish_refactor_preview_failure(
        failure_label,
        policy_passed,
        &policy_message,
        write_parse_refused,
    )
}

pub(super) fn build_refactor_preview(
    request: BuildRefactorPreviewRequest<'_>,
) -> Result<RefactorPreview> {
    let mut files = Vec::with_capacity(request.paths.len());
    let mut total_definitions = 0usize;
    let mut total_target_occurrences = 0usize;

    for file in request.paths {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, request.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        total_target_occurrences += matching_symbol_occurrences(&tree, request.to).len();
        let (rewritten, edits, definition_count) = match request.mode {
            RefactorPreviewMode::Symbol => {
                let raw_edits = matching_symbol_occurrences(&tree, request.from)
                    .into_iter()
                    .map(|occurrence| (occurrence.span, request.to.as_str().to_owned()))
                    .collect::<Vec<_>>();
                let rewritten = apply_byte_span_edits(&input.text, raw_edits.clone())?;
                (rewritten, refactor_preview_edits(&raw_edits), 0)
            }
            RefactorPreviewMode::Function => {
                let definitions = rename::shared::collect_callable_definition_renames(
                    &tree,
                    dialect,
                    request.from,
                    request.to,
                )?;
                let calls = rename::shared::collect_function_call_head_renames(
                    &tree,
                    dialect,
                    request.from,
                    request.to,
                )?;
                let raw_edits = definitions
                    .iter()
                    .chain(calls.iter())
                    .map(|edit| (edit.span, edit.replacement.clone()))
                    .collect::<Vec<_>>();
                let rewrite = apply_byte_span_edits(&input.text, raw_edits.clone())?;
                let definition_count = definitions.len();
                (
                    rewrite,
                    refactor_preview_edits(&raw_edits),
                    definition_count,
                )
            }
        };
        total_definitions += definition_count;

        let output_parse_ok = SyntaxTree::parse(&rewritten).is_ok();
        let changed = rewritten != input.text;
        let edit_count = edits.len();
        let preview = bounded_preview(&rewritten, request.max_preview_bytes);
        files.push(RefactorPreviewFile {
            path: file.clone(),
            dialect,
            changed,
            written: false,
            edit_count,
            edits,
            input_bytes: input.text.len(),
            output_bytes: rewritten.len(),
            output_parse_ok,
            input_hash: stable_text_hash(&input.text),
            output_hash: stable_text_hash(&rewritten),
            preview,
            rewritten,
        });
    }

    if request.mode == RefactorPreviewMode::Function && total_definitions == 0 {
        anyhow::bail!(
            "function '{}' was not found in callable definitions",
            request.from.as_str()
        );
    }

    let summary = RefactorPreviewSummary {
        file_count: files.len(),
        changed_file_count: files.iter().filter(|file| file.changed).count(),
        unchanged_file_count: files.iter().filter(|file| !file.changed).count(),
        written_file_count: 0,
        definition_count: total_definitions,
        target_occurrence_count: total_target_occurrences,
        edit_count: files.iter().map(|file| file.edit_count).sum(),
        parse_error_count: files.iter().filter(|file| !file.output_parse_ok).count(),
        all_outputs_parse: files.iter().all(|file| file.output_parse_ok),
    };
    let policy = evaluate_refactor_preview_policy(request.policy_options, &summary);

    Ok(RefactorPreview {
        workspace: request.workspace,
        mode: request.mode,
        from: request.from.as_str().to_owned(),
        to: request.to.as_str().to_owned(),
        write_requested: request.write,
        files,
        summary,
        policy,
    })
}

pub(super) fn write_refactor_preview(preview: &mut RefactorPreview) -> Result<()> {
    if !preview.write_requested {
        return Ok(());
    }

    for file in preview.files.iter_mut().filter(|file| file.changed) {
        fs::write(&file.path, &file.rewritten)
            .with_context(|| format!("failed to write {}", file.path.display()))?;
        file.written = true;
    }
    preview.summary.written_file_count = preview.files.iter().filter(|file| file.written).count();

    Ok(())
}

pub(super) fn finish_refactor_preview_failure(
    failure_label: &'static str,
    policy_passed: bool,
    policy_message: &str,
    write_parse_refused: bool,
) -> Result<()> {
    if !policy_passed {
        anyhow::bail!("{failure_label} policy failed: {policy_message}");
    }
    if write_parse_refused {
        anyhow::bail!("{failure_label} write refused because rewritten output failed to parse");
    }

    Ok(())
}
