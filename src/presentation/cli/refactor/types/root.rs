use super::super::super::*;

#[derive(Debug)]
pub(in crate::presentation::cli) struct RefactorRootReport {
    pub(in crate::presentation::cli) enforced: bool,
    pub(in crate::presentation::cli) path: Option<PathBuf>,
}

#[derive(Debug)]
pub(in crate::presentation::cli) struct RefactorRootGuard {
    pub(in crate::presentation::cli) root: PathBuf,
    pub(in crate::presentation::cli) canonical_root: PathBuf,
}
