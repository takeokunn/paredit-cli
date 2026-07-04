use super::super::super::*;

#[derive(Debug)]
pub(in crate::presentation::cli) struct RefactorVerification {
    pub(in crate::presentation::cli) operation: ApplicationRefactorOperation,
    pub(in crate::presentation::cli) phase: ApplicationVerificationPhase,
    pub(in crate::presentation::cli) symbol: String,
    pub(in crate::presentation::cli) new_symbol: Option<String>,
    pub(in crate::presentation::cli) passed: bool,
    pub(in crate::presentation::cli) checks: Vec<RefactorVerificationCheck>,
    pub(in crate::presentation::cli) before: RefactorPlanSummary,
    pub(in crate::presentation::cli) after: Option<RefactorPlanSummary>,
}
