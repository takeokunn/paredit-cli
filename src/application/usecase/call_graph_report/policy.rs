use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::sexpr::SymbolName;

use super::types::{CallGraphFile, CallGraphPolicy};

pub fn evaluate_call_graph_policy(
    reports: &[CallGraphFile],
    symbol: Option<&SymbolName>,
    fail_on_inbound_callers: bool,
    require_edges: Option<usize>,
    require_internal_edges: Option<usize>,
) -> CallGraphPolicy {
    let mut policy = CallGraphPolicy {
        fail_on_inbound_callers,
        require_edges,
        require_internal_edges,
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

    if fail_on_inbound_callers && !policy.inbound_callers.is_empty() {
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

    if let Some(required) = require_edges {
        if policy.edge_count < required {
            policy.passed = false;
            policy.violations.push(format!(
                "edge count {} is below required {}",
                policy.edge_count, required
            ));
        }
    }

    if let Some(required) = require_internal_edges {
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
