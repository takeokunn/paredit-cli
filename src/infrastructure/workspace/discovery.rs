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

    for root in &options.roots {
        collect_workspace_files(root, 0, options, &mut discovery, &mut seen)
            .with_context(|| format!("failed to scan {}", root.display()))?;
    }

    discovery.files.sort();
    Ok(discovery)
}

fn collect_workspace_files(
    path: &Path,
    depth: usize,
    options: &WorkspaceDiscoveryOptions,
    discovery: &mut WorkspaceDiscovery,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect {}", path.display()))?;

    if metadata.file_type().is_symlink() {
        discovery.skipped_symlink_count += 1;
        return Ok(());
    }

    if metadata.is_dir() {
        collect_workspace_directory(path, depth, options, discovery, seen)?;
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
        collect_workspace_files(&entry, depth + 1, options, discovery, seen)?;
    }

    Ok(())
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
