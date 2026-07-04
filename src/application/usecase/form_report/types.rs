use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Delimiter, ExpressionView, Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormReportRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub target: ExpressionView,
    pub include_source: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormReport {
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub span: ByteSpan,
    pub kind: FormKind,
    pub delimiter: Option<Delimiter>,
    pub head: Option<String>,
    pub definition_like: bool,
    pub child_count: usize,
    pub atom_count: usize,
    pub list_count: usize,
    pub max_depth: usize,
    pub symbols: Vec<FormSymbolReport>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormKind {
    Atom,
    List,
}

impl FormKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Atom => "atom",
            Self::List => "list",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSymbolReport {
    pub symbol: String,
    pub count: usize,
    pub first_span: ByteSpan,
}
