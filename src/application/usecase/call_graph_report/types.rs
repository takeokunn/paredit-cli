use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Path, SyntaxTree};

#[derive(Debug)]
pub struct CallGraphReportSource {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub tree: SyntaxTree,
}

#[derive(Debug)]
pub struct CallGraphReport {
    pub files: Vec<CallGraphFile>,
    pub nodes_by_name: BTreeMap<String, CallGraphNode>,
}

#[derive(Debug)]
pub struct CallGraphFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub definitions: Vec<CallGraphDefinitionItem>,
    pub edges: Vec<CallGraphEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallGraphDefinitionItem {
    pub name: Option<String>,
    pub category: DefinitionCategory,
    pub path: Path,
    pub span: ByteSpan,
    pub parameter_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CallGraphPolicy {
    pub inbound_callers: BTreeSet<String>,
    pub inbound_edge_count: usize,
    pub edge_count: usize,
    pub internal_edge_count: usize,
    pub fail_on_inbound_callers: bool,
    pub require_edges: Option<usize>,
    pub require_internal_edges: Option<usize>,
    pub passed: bool,
    pub violations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallGraphNode {
    pub name: String,
    pub definition_count: usize,
    pub categories: BTreeSet<DefinitionCategory>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallGraphEdge {
    pub caller: Option<String>,
    pub callee: String,
    pub path: String,
    pub argument_count: usize,
    pub span: ByteSpan,
    pub internal: bool,
    pub callee_categories: BTreeSet<DefinitionCategory>,
}
