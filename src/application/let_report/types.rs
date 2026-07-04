use crate::domain::sexpr::{ByteSpan, Path};

#[derive(Debug, Clone)]
pub struct LetFormReport {
    pub path: Path,
    pub form: String,
    pub span: ByteSpan,
    pub binding_style: &'static str,
    pub body_count: usize,
    pub inline_supported_by_inline_let: bool,
    pub bindings: Vec<LetBindingReport>,
}

#[derive(Debug, Clone)]
pub struct LetBindingReport {
    pub name: String,
    pub value: String,
    pub value_span: ByteSpan,
    pub reference_count: usize,
    pub can_inline_without_duplication: bool,
    pub risks: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct LetReportPolicyOptions {
    pub fail_on_duplicate_evaluation: bool,
    pub fail_on_unused_binding: bool,
    pub require_inlineable_bindings: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct LetReportPolicy {
    pub fail_on_duplicate_evaluation: bool,
    pub fail_on_unused_binding: bool,
    pub require_inlineable_bindings: Option<usize>,
    pub binding_count: usize,
    pub inlineable_binding_count: usize,
    pub unused_binding_count: usize,
    pub duplicate_evaluation_count: usize,
    pub passed: bool,
    pub violations: Vec<String>,
}
