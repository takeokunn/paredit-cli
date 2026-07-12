use std::fs;
use std::io;
use std::path::{Path as FsPath, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};

static STAGED_WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn write_files_with_rollback<I>(files: I) -> Result<()>
where
    I: IntoIterator<Item = (PathBuf, String)>,
{
    let staged = files
        .into_iter()
        .map(|(path, content)| stage_write_target(path, content))
        .collect::<Result<Vec<_>>>()?;
    let mut applied = Vec::with_capacity(staged.len());

    for target in staged {
        match apply_staged_write(&target) {
            Ok(()) => applied.push(target),
            Err(error) => {
                rollback_applied_writes(&applied)?;
                rollback_staged_write(&target)?;
                return Err(error)
                    .with_context(|| format!("failed to write {}", target.path.display()));
            }
        }
    }

    for target in applied {
        if target.existed {
            fs::remove_file(&target.backup_path).with_context(|| {
                format!("failed to clean up backup {}", target.backup_path.display())
            })?;
        }
    }

    Ok(())
}

pub(crate) fn write_file_with_rollback(path: PathBuf, content: String) -> Result<()> {
    write_files_with_rollback([(path, content)])
}

struct StagedWriteTarget {
    path: PathBuf,
    staged_path: PathBuf,
    backup_path: PathBuf,
    existed: bool,
}

fn stage_write_target(path: PathBuf, content: String) -> Result<StagedWriteTarget> {
    let staged_path = sibling_staging_path(&path, "tmp");
    let backup_path = sibling_staging_path(&path, "bak");
    let existed = path.exists();

    fs::write(&staged_path, content)
        .with_context(|| format!("failed to stage {}", staged_path.display()))?;
    if existed {
        let permissions = fs::metadata(&path)
            .with_context(|| format!("failed to stat {}", path.display()))?
            .permissions();
        fs::set_permissions(&staged_path, permissions)
            .with_context(|| format!("failed to copy permissions to {}", staged_path.display()))?;
    }

    Ok(StagedWriteTarget {
        path,
        staged_path,
        backup_path,
        existed,
    })
}

fn apply_staged_write(target: &StagedWriteTarget) -> io::Result<()> {
    if target.existed {
        fs::rename(&target.path, &target.backup_path)?;
    }

    match fs::rename(&target.staged_path, &target.path) {
        Ok(()) => Ok(()),
        Err(error) => {
            if target.existed {
                let _ = fs::rename(&target.backup_path, &target.path);
            }
            Err(error)
        }
    }
}

fn rollback_staged_write(target: &StagedWriteTarget) -> io::Result<()> {
    if target.staged_path.exists() {
        fs::remove_file(&target.staged_path)?;
    }

    if target.existed && target.backup_path.exists() {
        if target.path.exists() {
            let _ = fs::remove_file(&target.path);
        }
        fs::rename(&target.backup_path, &target.path)?;
    }

    Ok(())
}

fn rollback_applied_writes(applied: &[StagedWriteTarget]) -> io::Result<()> {
    for target in applied.iter().rev() {
        if target.path.exists() {
            fs::remove_file(&target.path)?;
        }

        if target.existed {
            fs::rename(&target.backup_path, &target.path)?;
        }
    }

    Ok(())
}

fn sibling_staging_path(path: &FsPath, suffix: &str) -> PathBuf {
    let counter = STAGED_WRITE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("paredit");
    path.with_file_name(format!(".{file_name}.paredit-{suffix}-{pid}-{counter}"))
}
