use super::*;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
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
        exclude: Vec::new(),
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
        exclude: Vec::new(),
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

#[test]
fn discovery_excludes_files_and_directory_subtrees_by_component() -> Result<()> {
    let root = unique_temp_dir("exclude-paths");
    let excluded_dir = root.join("vendor");
    let prefix_sibling = root.join("vendor-copy");
    let excluded_file = root.join("excluded.lisp");
    fs::create_dir_all(&excluded_dir)?;
    fs::create_dir_all(&prefix_sibling)?;
    fs::write(root.join("keep.lisp"), "(defun keep () nil)")?;
    fs::write(&excluded_file, "(defun excluded () nil)")?;
    fs::write(excluded_dir.join("nested.lisp"), "(defun nested () nil)")?;
    fs::write(prefix_sibling.join("keep.lisp"), "(defun sibling () nil)")?;

    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: vec![root.clone()],
        include_unknown: false,
        include_hidden: false,
        include_generated: false,
        max_depth: None,
        exclude: vec![excluded_file, excluded_dir],
    })?;

    assert_eq!(
        discovery.files,
        vec![root.join("keep.lisp"), prefix_sibling.join("keep.lisp")]
    );
    assert_eq!(discovery.skipped_excluded_count, 2);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn discovery_can_exclude_an_explicit_root() -> Result<()> {
    let root = unique_temp_dir("exclude-root");
    fs::create_dir_all(&root)?;
    fs::write(root.join("nested.lisp"), "(defun nested () nil)")?;

    let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
        roots: vec![root.clone()],
        include_unknown: false,
        include_hidden: false,
        include_generated: false,
        max_depth: None,
        exclude: vec![root.clone()],
    })?;

    assert!(discovery.files.is_empty());
    assert_eq!(discovery.skipped_excluded_count, 1);

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
