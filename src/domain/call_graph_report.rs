use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Path, SymbolName, SyntaxTree};

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

#[derive(Debug, Clone, Copy)]
pub struct CallGraphPolicyOptions {
    fail_on_inbound_callers: bool,
    require_edges: Option<usize>,
    require_internal_edges: Option<usize>,
}

impl CallGraphPolicyOptions {
    pub fn new(
        fail_on_inbound_callers: bool,
        require_edges: Option<usize>,
        require_internal_edges: Option<usize>,
    ) -> Result<Self, String> {
        Self::validate_threshold("require-edges", require_edges)?;
        Self::validate_threshold("require-internal-edges", require_internal_edges)?;

        Ok(Self {
            fail_on_inbound_callers,
            require_edges,
            require_internal_edges,
        })
    }

    fn validate_threshold(label: &str, value: Option<usize>) -> Result<(), String> {
        if matches!(value, Some(0)) {
            return Err(format!("{label} must be greater than zero"));
        }
        Ok(())
    }

    pub const fn fail_on_inbound_callers(self) -> bool {
        self.fail_on_inbound_callers
    }

    pub const fn require_edges(self) -> Option<usize> {
        self.require_edges
    }

    pub const fn require_internal_edges(self) -> Option<usize> {
        self.require_internal_edges
    }
}

pub fn evaluate_call_graph_policy(
    reports: &[CallGraphFile],
    symbol: Option<&SymbolName>,
    options: CallGraphPolicyOptions,
) -> CallGraphPolicy {
    let mut policy = CallGraphPolicy {
        fail_on_inbound_callers: options.fail_on_inbound_callers(),
        require_edges: options.require_edges(),
        require_internal_edges: options.require_internal_edges(),
        passed: true,
        ..CallGraphPolicy::default()
    };

    for edge in reports.iter().flat_map(|report| &report.edges) {
        if symbol
            .map(|symbol| {
                edge.caller
                    .as_deref()
                    .is_some_and(|caller| common_lisp_symbol_reference_eq(caller, symbol.as_str()))
                    || common_lisp_symbol_reference_eq(&edge.callee, symbol.as_str())
            })
            .unwrap_or(true)
        {
            policy.edge_count += 1;
            if edge.internal {
                policy.internal_edge_count += 1;
            }
        }

        if let Some(symbol) = symbol {
            let callee_matches = common_lisp_symbol_reference_eq(&edge.callee, symbol.as_str());
            let caller_matches = edge
                .caller
                .as_deref()
                .is_some_and(|caller| common_lisp_symbol_reference_eq(caller, symbol.as_str()));
            if callee_matches && !caller_matches {
                policy.inbound_edge_count += 1;
                if let Some(caller) = &edge.caller {
                    policy.inbound_callers.insert(caller.clone());
                }
            }
        }
    }

    if options.fail_on_inbound_callers() && !policy.inbound_callers.is_empty() {
        policy.passed = false;
        policy.violations.push(format!(
            "focused symbol has inbound callers: {}",
            policy
                .inbound_callers
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if let Some(required) = options.require_edges() {
        if policy.edge_count < required {
            policy.passed = false;
            policy.violations.push(format!(
                "edge count {} is below required {}",
                policy.edge_count, required
            ));
        }
    }

    if let Some(required) = options.require_internal_edges() {
        if policy.internal_edge_count < required {
            policy.passed = false;
            policy.violations.push(format!(
                "internal edge count {} is below required {}",
                policy.internal_edge_count, required
            ));
        }
    }

    policy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_thresholds() {
        assert!(CallGraphPolicyOptions::new(true, Some(1), Some(2)).is_ok());
        assert_eq!(
            CallGraphPolicyOptions::new(false, Some(0), None).unwrap_err(),
            "require-edges must be greater than zero"
        );
        assert_eq!(
            CallGraphPolicyOptions::new(false, None, Some(0)).unwrap_err(),
            "require-internal-edges must be greater than zero"
        );
    }
}
