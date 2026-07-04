use std::path::PathBuf;

use crate::application::call_graph_report::CallGraphEdge;
use crate::application::refactor::plan::RefactorRiskLevel;
use crate::application::signature_report::SignatureCallItem;
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, SyntaxTree};

#[derive(Debug)]
pub struct ImpactReportSource {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub tree: SyntaxTree,
}

#[derive(Debug)]
pub struct ImpactReportFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub package: Option<String>,
    pub definitions: Vec<ImpactDefinitionItem>,
    pub references: Vec<ImpactSymbolOccurrence>,
    pub calls: Vec<SignatureCallItem>,
    pub inbound_edges: Vec<CallGraphEdge>,
    pub outbound_edges: Vec<CallGraphEdge>,
    pub non_call_reference_count: usize,
}

#[derive(Debug, Clone)]
pub struct ImpactDefinitionItem {
    pub path: String,
    pub span: ByteSpan,
    pub head: String,
    pub name: Option<String>,
    pub category: DefinitionCategory,
    pub parameter_count: Option<usize>,
    pub body_form_count: Option<usize>,
    pub package: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImpactSymbolOccurrence {
    pub path: String,
    pub span: ByteSpan,
    pub context: Option<ImpactSymbolOccurrenceContext>,
}

#[derive(Debug, Clone)]
pub struct ImpactSymbolOccurrenceContext {
    pub path: String,
    pub span: ByteSpan,
    pub head: Option<String>,
    pub definition_like: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactRiskLevel {
    Info,
    Warning,
    Error,
}

impl ImpactRiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl From<ImpactRiskLevel> for RefactorRiskLevel {
    fn from(value: ImpactRiskLevel) -> Self {
        match value {
            ImpactRiskLevel::Info => Self::Info,
            ImpactRiskLevel::Warning => Self::Warning,
            ImpactRiskLevel::Error => Self::Error,
        }
    }
}

#[derive(Debug)]
pub struct ImpactRisk {
    pub level: ImpactRiskLevel,
    pub code: &'static str,
    pub message: String,
    pub count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct ImpactReportPolicyOptions {
    pub fail_on_risk_level: Option<ImpactRiskLevel>,
    pub require_definitions: Option<usize>,
    pub require_references: Option<usize>,
    pub require_calls: Option<usize>,
}

#[derive(Debug)]
pub struct ImpactReportPolicy {
    pub fail_on_risk_level: Option<ImpactRiskLevel>,
    pub require_definitions: Option<usize>,
    pub require_references: Option<usize>,
    pub require_calls: Option<usize>,
    pub definition_count: usize,
    pub reference_count: usize,
    pub call_count: usize,
    pub inbound_edge_count: usize,
    pub non_call_reference_count: usize,
    pub signature_mismatch_count: usize,
    pub risk_level: ImpactRiskLevel,
    pub passed: bool,
    pub violations: Vec<String>,
}
