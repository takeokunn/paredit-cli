//! Workspace filesystem discovery adapters.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::dialect::Dialect;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct WorkspaceDiscoveryOptions {
    pub roots: Vec<PathBuf>,
    pub include_unknown: bool,
    pub include_hidden: bool,
    pub include_generated: bool,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct WorkspaceDiscovery {
    pub files: Vec<PathBuf>,
    pub skipped_unknown_count: usize,
    pub skipped_hidden_count: usize,
    pub skipped_generated_count: usize,
    pub skipped_symlink_count: usize,
}

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

        return Ok(());
    }

    if !metadata.is_file() {
        return Ok(());
    }

    if !options.include_hidden && is_hidden_workspace_path(path) {
        discovery.skipped_hidden_count += 1;
        return Ok(());
    }

    let dialect = Dialect::detect(Some(path), None);
    if dialect == Dialect::Unknown && !options.include_unknown {
        discovery.skipped_unknown_count += 1;
        return Ok(());
    }

    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if seen.insert(canonical) {
        discovery.files.push(path.to_path_buf());
    }

    Ok(())
}

fn is_hidden_workspace_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn is_generated_workspace_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git"
                    | ".direnv"
                    | ".devenv"
                    | "build"
                    | "coverage"
                    | "dist"
                    | "node_modules"
                    | "result"
                    | "target"
                    | "vendor"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn discovery_skips_unknown_hidden_and_generated_paths_by_default() -> Result<()> {
        let root = unique_temp_dir("default-skips");
        fs::create_dir_all(root.join(".hidden"))?;
        fs::create_dir_all(root.join("target"))?;
        fs::write(root.join("main.lisp"), "(defun main () nil)")?;
        fs::write(root.join("README.md"), "not lisp")?;
        fs::write(
            root.join(".hidden").join("secret.lisp"),
            "(defun secret () nil)",
        )?;
        fs::write(
            root.join("target").join("generated.lisp"),
            "(defun generated () nil)",
        )?;

        let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
            roots: vec![root.clone()],
            include_unknown: false,
            include_hidden: false,
            include_generated: false,
            max_depth: None,
        })?;

        assert_eq!(discovery.files, vec![root.join("main.lisp")]);
        assert_eq!(discovery.skipped_unknown_count, 1);
        assert_eq!(discovery.skipped_hidden_count, 1);
        assert_eq!(discovery.skipped_generated_count, 1);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn discovery_can_include_unknown_hidden_and_generated_paths() -> Result<()> {
        let root = unique_temp_dir("include-all");
        fs::create_dir_all(root.join(".hidden"))?;
        fs::create_dir_all(root.join("target"))?;
        fs::write(root.join("main.lisp"), "(defun main () nil)")?;
        fs::write(root.join("README.md"), "not lisp")?;
        fs::write(
            root.join(".hidden").join("secret.el"),
            "(defun secret () nil)",
        )?;
        fs::write(
            root.join("target").join("generated.scm"),
            "(define generated #t)",
        )?;

        let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
            roots: vec![root.clone()],
            include_unknown: true,
            include_hidden: true,
            include_generated: true,
            max_depth: None,
        })?;

        assert_eq!(
            discovery.files,
            vec![
                root.join(".hidden").join("secret.el"),
                root.join("README.md"),
                root.join("main.lisp"),
                root.join("target").join("generated.scm"),
            ]
        );
        assert_eq!(discovery.skipped_unknown_count, 0);
        assert_eq!(discovery.skipped_hidden_count, 0);
        assert_eq!(discovery.skipped_generated_count, 0);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before UNIX_EPOCH")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "paredit-cli-workspace-{name}-{}-{nanos}",
            std::process::id()
        ))
    }
}
