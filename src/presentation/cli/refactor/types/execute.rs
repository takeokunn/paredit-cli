use super::preview::RefactorPreview;
use super::verification::RefactorVerification;

#[derive(Debug)]
pub(in crate::presentation::cli) struct WorkspaceRefactorExecute {
    pub(in crate::presentation::cli) preview: RefactorPreview,
    pub(in crate::presentation::cli) post_verification: Option<RefactorVerification>,
}
