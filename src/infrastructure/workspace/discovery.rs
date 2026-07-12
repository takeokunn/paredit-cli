use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::filters::{is_generated_workspace_path, is_hidden_workspace_path};
use super::types::{WorkspaceDiscovery, WorkspaceDiscoveryOptions};
use crate::domain::dialect::Dialect;

pub fn discover_workspace_files(options: &WorkspaceDiscoveryOptions) -> Result<WorkspaceDiscovery> {
    let mut discovery = WorkspaceDiscovery::default();
    let mut seen = BTreeSet::new();
    let excludes = normalized_excludes(&options.exclude)?;

    for root in &options.roots {
        collect_workspace_files(root, 0, options, &excludes, &mut discovery, &mut seen)
            .with_context(|| format!("failed to scan {}", root.display()))?;
    }

    discovery.files.sort();
    Ok(discovery)
}

fn collect_workspace_files(
    path: &Path,
    depth: usize,
    options: &WorkspaceDiscoveryOptions,
    excludes: &[PathBuf],
    discovery: &mut WorkspaceDiscovery,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    if is_excluded(path, excludes)? {
        discovery.skipped_excluded_count += 1;
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect {}", path.display()))?;

    if metadata.file_type().is_symlink() {
        discovery.skipped_symlink_count += 1;
        return Ok(());
    }

    if metadata.is_dir() {
        collect_workspace_directory(path, depth, options, excludes, discovery, seen)?;
        return Ok(());
    }

    if metadata.is_file() {
        collect_workspace_file(path, options, discovery, seen);
    }

    Ok(())
}

fn collect_workspace_directory(
    path: &Path,
    depth: usize,
    options: &WorkspaceDiscoveryOptions,
    excludes: &[PathBuf],
    discovery: &mut WorkspaceDiscovery,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    if !options.include_hidden && is_hidden_workspace_path(path) {
        discovery.skipped_hidden_count += 1;
        return Ok(());
    }

    if !options.include_generated && is_generated_workspace_path(path) {
        discovery.skipped_generated_count += 1;
        return Ok(());
    }

    if options
        .max_depth
        .is_some_and(|max_depth| depth >= max_depth)
    {
        return Ok(());
    }

    let mut entries = fs::read_dir(path)
        .with_context(|| format!("failed to list {}", path.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to list {}", path.display()))?;
    entries.sort();

    for entry in entries {
        collect_workspace_files(&entry, depth + 1, options, excludes, discovery, seen)?;
    }

    Ok(())
}

fn normalized_excludes(excludes: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let current_dir = std::env::current_dir().context("failed to resolve current directory")?;
    let mut normalized = excludes
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() {
                path.clone()
            } else {
                current_dir.join(path)
            };
            fs::canonicalize(&absolute).unwrap_or(absolute)
        })
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn is_excluded(path: &Path, excludes: &[PathBuf]) -> Result<bool> {
    if excludes.is_empty() {
        return Ok(false);
    }
    let current_dir = std::env::current_dir().context("failed to resolve current directory")?;
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };
    let normalized = fs::canonicalize(&absolute).unwrap_or(absolute);
    Ok(excludes
        .iter()
        .any(|exclude| normalized.starts_with(exclude)))
}

fn collect_workspace_file(
    path: &Path,
    options: &WorkspaceDiscoveryOptions,
    discovery: &mut WorkspaceDiscovery,
    seen: &mut BTreeSet<PathBuf>,
) {
    if !options.include_hidden && is_hidden_workspace_path(path) {
        discovery.skipped_hidden_count += 1;
        return;
    }

    if !options.include_generated && is_generated_workspace_path(path) {
        discovery.skipped_generated_count += 1;
        return;
    }

    let dialect = Dialect::detect(Some(path), None);
    if dialect == Dialect::Unknown && !options.include_unknown {
        discovery.skipped_unknown_count += 1;
        return;
    }

    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if seen.insert(canonical) {
        discovery.files.push(path.to_path_buf());
    }
}
