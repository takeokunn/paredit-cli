use super::super::super::super::*;
use super::super::super::types::preview::RefactorPreview;

pub(in crate::presentation::cli::refactor::workflow) fn write_refactor_preview(
    preview: &mut RefactorPreview,
) -> Result<()> {
    let candidates = preview
        .files
        .iter()
        .map(|file| RefactorWriteCandidate {
            changed: file.changed,
            output_parse_ok: file.output_parse_ok,
        })
        .collect::<Vec<_>>();
    let write_plan = build_refactor_write_plan(preview.write_requested, &candidates);

    if !write_plan.write_requested {
        return Ok(());
    }

    if let Some(refusal) = write_plan.refusal {
        match refusal {
            RefactorWriteRefusal::UnparsableOutputs { count } => anyhow::bail!(
                "refactor write refused because {count} rewritten output file(s) failed to parse"
            ),
        }
    }

    for index in write_plan.writable_indexes {
        let file = &mut preview.files[index];
        fs::write(&file.path, &file.rewritten)
            .with_context(|| format!("failed to write {}", file.path.display()))?;
        file.written = true;
    }
    preview.summary.written_file_count = preview.files.iter().filter(|file| file.written).count();

    Ok(())
}
