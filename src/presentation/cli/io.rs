use std::fs;
use std::io::{self, ErrorKind, IsTerminal, Read};
use std::path::{Path as FsPath, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};

use super::{DialectArg, SourceInput};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ParseError, SyntaxTree};

static STAGED_WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn read_input(file: Option<PathBuf>) -> Result<SourceInput> {
    match file {
        Some(path) => {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            Ok(SourceInput {
                text,
                file: Some(path),
            })
        }
        None => {
            let mut stdin = io::stdin();
            if stdin.is_terminal() {
                anyhow::bail!(
                    "no input: pass --file <path> or pipe source into stdin \
                     (refusing to wait on an interactive terminal)"
                );
            }
            let mut text = String::new();
            stdin
                .read_to_string(&mut text)
                .context("failed to read stdin")?;
            Ok(SourceInput { text, file: None })
        }
    }
}

pub(crate) fn read_input_and_dialect(
    file: Option<PathBuf>,
    explicit: Option<DialectArg>,
) -> Result<(SourceInput, Dialect)> {
    let input = read_input(file)?;
    let dialect = super::detect_dialect(&input, explicit);
    Ok((input, dialect))
}

pub(crate) fn read_input_dialect_and_tree(
    file: Option<PathBuf>,
    explicit: Option<DialectArg>,
) -> Result<(SourceInput, Dialect, SyntaxTree)> {
    let (input, dialect) = read_input_and_dialect(file, explicit)?;
    let tree = parse_document(&input)?;
    Ok((input, dialect, tree))
}

/// Parses a source document, naming the input and the error's line/column in
/// the context. The underlying [`ParseError`] keeps the raw byte offset,
/// which feeds directly into `--at`.
pub(crate) fn parse_document(input: &SourceInput) -> Result<SyntaxTree> {
    SyntaxTree::parse(&input.text).map_err(|error| {
        let location = parse_error_line_column(&input.text, &error);
        let source = match input.file.as_deref() {
            Some(path) => path.display().to_string(),
            None => "stdin".to_owned(),
        };
        anyhow::Error::new(error).context(format!("failed to parse {source} ({location})"))
    })
}

fn parse_error_line_column(text: &str, error: &ParseError) -> String {
    let position = match error {
        ParseError::UnexpectedClose { position, .. }
        | ParseError::MismatchedClose { position, .. } => *position,
        ParseError::UnclosedList(position)
        | ParseError::UnterminatedString(position)
        | ParseError::UnterminatedBlockComment(position)
        | ParseError::UnterminatedSymbol(position) => *position,
    };
    let clamped = position.min(text.len());
    let line = text[..clamped]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let column = match text[..clamped].rfind('\n') {
        Some(newline) => clamped - newline,
        None => clamped + 1,
    };
    format!("line {line}, column {column}")
}

pub(crate) fn read_file_or_empty(path: &PathBuf) -> Result<(SourceInput, bool)> {
    match fs::read_to_string(path) {
        Ok(text) => Ok((
            SourceInput {
                text,
                file: Some(path.clone()),
            },
            true,
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok((
            SourceInput {
                text: String::new(),
                file: Some(path.clone()),
            },
            false,
        )),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}

pub(crate) fn write_files_with_rollback<I>(files: I) -> Result<()>
where
    I: IntoIterator<Item = (PathBuf, String)>,
{
    let files = files.into_iter().collect::<Vec<_>>();
    // Central invariant: paredit never persists an unbalanced document, no
    // matter which command produced the rewrite.
    for (path, content) in &files {
        SyntaxTree::parse(content).with_context(|| {
            format!(
                "refusing to write {}: rewritten source does not reparse",
                path.display()
            )
        })?;
    }

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
