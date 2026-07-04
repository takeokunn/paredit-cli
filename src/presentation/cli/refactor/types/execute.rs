use super::preview::RefactorPreview;
use super::verification::RefactorVerification;
use crate::application::refactor::execute::RefactorExecuteDecision;

#[derive(Debug)]
pub(in crate::presentation::cli) struct WorkspaceRefactorExecute {
    pub(in crate::presentation::cli) preview: RefactorPreview,
    pub(in crate::presentation::cli) preflight_decision: RefactorExecuteDecision,
    pub(in crate::presentation::cli) execute_decision: RefactorExecuteDecision,
    pub(in crate::presentation::cli) pre_verification: Option<RefactorVerification>,
    pub(in crate::presentation::cli) post_verification: Option<RefactorVerification>,
}
