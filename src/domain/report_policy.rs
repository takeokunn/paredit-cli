#[derive(Debug, Clone, Copy)]
pub struct ImpactReportPolicyOptions {
    pub fail_on_risk_level: Option<crate::application::refactor::plan::RefactorRiskLevel>,
    pub require_definitions: Option<usize>,
    pub require_references: Option<usize>,
    pub require_calls: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct LetReportPolicyOptions {
    pub fail_on_duplicate_evaluation: bool,
    pub fail_on_unused_binding: bool,
    pub require_inlineable_bindings: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct UnusedDefinitionPolicyOptions {
    pub fail_on_unused: bool,
    pub require_unused_definitions: Option<usize>,
}
