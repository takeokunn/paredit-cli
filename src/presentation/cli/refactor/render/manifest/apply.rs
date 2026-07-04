use super::super::super::super::*;
use super::super::super::types::apply::RefactorApplyResult;

pub(in crate::presentation::cli) fn print_refactor_apply_result(
    result: &RefactorApplyResult,
    output: OutputFormat,
) -> Result<()> {
    match output {
        OutputFormat::Text => {
            println!("manifest_path\t{}", result.manifest.path.display());
            println!("manifest_hash\t{}", result.manifest.hash);
            println!("mode\t{}", result.manifest.mode);
            println!("from\t{}", result.manifest.from);
            println!("to\t{}", result.manifest.to);
            println!("root_enforced\t{}", result.root.enforced);
            if let Some(path) = &result.root.path {
                println!("root\t{}", path.display());
            }
            println!("write_requested\t{}", result.write_requested);
            println!("applied\t{}", result.summary.applied);
            println!("files\t{}", result.summary.file_count);
            println!("changed_file_count\t{}", result.summary.changed_file_count);
            println!("written_file_count\t{}", result.summary.written_file_count);
            println!("edit_count\t{}", result.summary.edit_count);
            println!("stale_file_count\t{}", result.summary.stale_file_count);
            println!(
                "output_hash_mismatch_count\t{}",
                result.summary.output_hash_mismatch_count
            );
            println!("parse_error_count\t{}", result.summary.parse_error_count);
            println!(
                "manifest_flag_mismatch_count\t{}",
                result.summary.manifest_flag_mismatch_count
            );
            for file in &result.files {
                println!(
                    "file\t{}\tchanged={}\texpected_changed={}\twritten={}\tedits={}\tinput_hash_matches={}\toutput_hash_matches={}\tparse={}\texpected_parse={}\tmanifest_flags_match={}",
                    file.path.display(),
                    file.changed,
                    file.expected_changed,
                    file.written,
                    file.edit_count,
                    file.input_hash_matches,
                    file.output_hash_matches,
                    file.output_parse_ok,
                    file.expected_output_parse_ok,
                    file.manifest_flags_match
                );
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "manifest": {
                    "path": result.manifest.path.display().to_string(),
                    "hash": result.manifest.hash.as_str(),
                    "mode": result.manifest.mode.as_str(),
                    "from": result.manifest.from.as_str(),
                    "to": result.manifest.to.as_str(),
                },
                "root": {
                    "enforced": result.root.enforced,
                    "path": result.root.path.as_ref().map(|path| path.display().to_string()),
                },
                "write_requested": result.write_requested,
                "summary": {
                    "file_count": result.summary.file_count,
                    "changed_file_count": result.summary.changed_file_count,
                    "written_file_count": result.summary.written_file_count,
                    "edit_count": result.summary.edit_count,
                    "stale_file_count": result.summary.stale_file_count,
                    "output_hash_mismatch_count": result.summary.output_hash_mismatch_count,
                    "parse_error_count": result.summary.parse_error_count,
                    "manifest_flag_mismatch_count": result.summary.manifest_flag_mismatch_count,
                    "applied": result.summary.applied,
                },
                "files": result.files
                    .iter()
                    .map(|file| json!({
                        "path": file.path.display().to_string(),
                        "changed": file.changed,
                        "expected_changed": file.expected_changed,
                        "written": file.written,
                        "edit_count": file.edit_count,
                        "input_hash": file.input_hash.as_str(),
                        "output_hash": file.output_hash.as_str(),
                        "expected_input_hash": file.expected_input_hash.as_str(),
                        "expected_output_hash": file.expected_output_hash.as_str(),
                        "input_hash_matches": file.input_hash_matches,
                        "output_hash_matches": file.output_hash_matches,
                        "output_parse_ok": file.output_parse_ok,
                        "expected_output_parse_ok": file.expected_output_parse_ok,
                        "manifest_flags_match": file.manifest_flags_match,
                    }))
                    .collect::<Vec<_>>(),
            }))?
        ),
    }

    Ok(())
}
