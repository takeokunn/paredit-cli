use super::super::super::super::*;
use super::super::super::args::RefactorApplyArgs;
use super::super::super::manifest::io::read_refactor_manifest_file;
use super::super::super::manifest::parse::parse_refactor_apply_manifest;
use super::super::super::manifest::root::resolve_refactor_manifest_path;
use super::super::super::manifest::validation::validate_manifest_edits;
use super::super::super::render::print_refactor_apply_result;
use super::super::super::types::apply::{
    RefactorApplyFileResult, RefactorApplyResult, RefactorApplySummary,
};
use super::super::super::types::manifest::RefactorApplyManifestHeader;
use super::super::super::types::root::{RefactorRootGuard, RefactorRootReport};
use crate::presentation::cli::shared::write_files_with_rollback;

pub(in crate::presentation::cli) fn refactor_apply(args: RefactorApplyArgs) -> Result<()> {
    let loaded_manifest =
        read_refactor_manifest_file(&args.manifest, args.expect_manifest_hash.as_deref())?;
    let manifest = parse_refactor_apply_manifest(&loaded_manifest.value)?;
    let root_guard = args
        .root
        .as_deref()
        .map(RefactorRootGuard::new)
        .transpose()?;

    let mut files = Vec::with_capacity(manifest.files.len());
    let mut rewritten_outputs = Vec::with_capacity(manifest.files.len());

    for file in &manifest.files {
        let resolved_path = resolve_refactor_manifest_path(&file.path, root_guard.as_ref())?;
        let input = fs::read_to_string(&resolved_path)
            .with_context(|| format!("failed to read {}", resolved_path.display()))?;
        let input_hash = stable_text_hash(&input);
        let input_hash_matches = input_hash == file.input_hash;
        let edits = file
            .edits
            .iter()
            .map(|edit| (edit.span, edit.replacement.clone()))
            .collect::<Vec<_>>();
        validate_manifest_edits(&input, &edits)
            .with_context(|| format!("manifest edits are invalid for {}", file.path.display()))?;
        let rewritten = apply_byte_span_edits(&input, edits)?;
        let output_hash = stable_text_hash(&rewritten);
        let output_hash_matches = output_hash == file.output_hash;
        let output_parse_ok = SyntaxTree::parse(&rewritten).is_ok();
        let changed = rewritten != input;
        let manifest_flags_match =
            changed == file.changed && output_parse_ok == file.output_parse_ok;

        files.push(RefactorApplyFileResult {
            path: file.path.clone(),
            changed,
            expected_changed: file.changed,
            written: false,
            edit_count: file.edits.len(),
            input_hash,
            output_hash,
            expected_input_hash: file.input_hash.clone(),
            expected_output_hash: file.output_hash.clone(),
            input_hash_matches,
            output_hash_matches,
            output_parse_ok,
            expected_output_parse_ok: file.output_parse_ok,
            manifest_flags_match,
        });
        rewritten_outputs.push((resolved_path, rewritten));
    }

    let stale_file_count = files.iter().filter(|file| !file.input_hash_matches).count();
    let output_hash_mismatch_count = files
        .iter()
        .filter(|file| !file.output_hash_matches)
        .count();
    let parse_error_count = files.iter().filter(|file| !file.output_parse_ok).count();
    let manifest_flag_mismatch_count = files
        .iter()
        .filter(|file| !file.manifest_flags_match)
        .count();
    let can_apply = manifest.policy_passed
        && manifest.all_outputs_parse
        && stale_file_count == 0
        && output_hash_mismatch_count == 0
        && parse_error_count == 0
        && manifest_flag_mismatch_count == 0;

    if args.write && can_apply {
        let mut written_outputs = Vec::new();
        let mut written_indexes = Vec::new();
        for (index, (resolved_path, rewritten)) in rewritten_outputs.iter().enumerate() {
            if files.get(index).is_some_and(|file| file.changed) {
                written_outputs.push((resolved_path.clone(), rewritten.clone()));
                written_indexes.push(index);
            }
        }

        write_files_with_rollback(written_outputs)?;

        for index in written_indexes {
            if let Some(file) = files.get_mut(index) {
                file.written = true;
            }
        }
    }

    let changed_files = files
        .iter()
        .filter(|file| file.changed)
        .map(|file| file.path.display().to_string())
        .collect::<Vec<_>>();
    let summary = RefactorApplySummary {
        file_count: files.len(),
        changed_file_count: changed_files.len(),
        changed_files,
        written_file_count: files.iter().filter(|file| file.written).count(),
        edit_count: files.iter().map(|file| file.edit_count).sum(),
        stale_file_count,
        output_hash_mismatch_count,
        parse_error_count,
        manifest_flag_mismatch_count,
        applied: args.write && can_apply,
    };
    let result = RefactorApplyResult {
        manifest: RefactorApplyManifestHeader {
            path: args.manifest,
            hash: loaded_manifest.hash,
            mode: manifest.mode,
            from: manifest.from,
            to: manifest.to,
        },
        root: RefactorRootReport::from_guard(root_guard.as_ref()),
        write_requested: args.write,
        manifest_policy_passed: manifest.policy_passed,
        manifest_outputs_parse: manifest.all_outputs_parse,
        files,
        summary,
    };

    print_refactor_apply_result(&result, args.output)?;

    if !can_apply {
        anyhow::bail!(
            "refactor apply validation failed: manifest_policy_passed={}, manifest_outputs_parse={}, stale_files={}, output_hash_mismatches={}, parse_errors={}, manifest_flag_mismatches={}",
            result.manifest_policy_passed,
            result.manifest_outputs_parse,
            result.summary.stale_file_count,
            result.summary.output_hash_mismatch_count,
            result.summary.parse_error_count,
            result.summary.manifest_flag_mismatch_count
        );
    }

    Ok(())
}
