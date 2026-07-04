use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, ExpressionView, Path, SymbolName};

#[derive(Debug, Clone)]
pub struct InlineLetRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub target: ExpressionView,
    pub allow_duplicate_evaluation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineLetPlan {
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub let_span: ByteSpan,
    pub binding_name: SymbolName,
    pub binding_value: String,
    pub body_count: usize,
    pub reference_count: usize,
    pub replacement: String,
    pub rewritten: String,
    pub changed: bool,
}
