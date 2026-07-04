use super::super::super::*;
use super::super::types::root::{RefactorRootGuard, RefactorRootReport};

impl RefactorRootGuard {
    pub(in crate::presentation::cli) fn new(root: &FsPath) -> Result<Self> {
        let canonical_root = fs::canonicalize(root)
            .with_context(|| format!("failed to canonicalize refactor root {}", root.display()))?;
        if !canonical_root.is_dir() {
            anyhow::bail!(
                "refactor root {} is not a directory",
                canonical_root.display()
            );
        }
        Ok(Self {
            root: root.to_path_buf(),
            canonical_root,
        })
    }

    pub(in crate::presentation::cli) fn resolve_manifest_path(
        &self,
        path: &FsPath,
    ) -> Result<PathBuf> {
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        let canonical_path = fs::canonicalize(&resolved)
            .with_context(|| format!("failed to canonicalize manifest path {}", path.display()))?;
        if !canonical_path.starts_with(&self.canonical_root) {
            anyhow::bail!(
                "manifest path {} is outside refactor root {}",
                path.display(),
                self.canonical_root.display()
            );
        }
        Ok(resolved)
    }
}

impl RefactorRootReport {
    pub(in crate::presentation::cli) fn from_guard(root_guard: Option<&RefactorRootGuard>) -> Self {
        match root_guard {
            Some(root_guard) => Self {
                enforced: true,
                path: Some(root_guard.canonical_root.clone()),
            },
            None => Self {
                enforced: false,
                path: None,
            },
        }
    }
}

pub(in crate::presentation::cli) fn resolve_refactor_manifest_path(
    path: &FsPath,
    root_guard: Option<&RefactorRootGuard>,
) -> Result<PathBuf> {
    match root_guard {
        Some(root_guard) => root_guard.resolve_manifest_path(path),
        None => Ok(path.to_path_buf()),
    }
}
