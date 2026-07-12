use std::path::PathBuf;

use crate::application::usecase::signature_report::SignatureCallItem;
use crate::domain::call_graph_report::CallGraphEdge;
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
pub use crate::domain::impact_report::{ImpactReportPolicy, ImpactRisk, ImpactRiskLevel};
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
    pub parameter_arity: Option<(usize, Option<usize>)>,
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
