use std::collections::{BTreeMap, BTreeSet};

use crate::application::usecase::call_report::CallReportItem;
use crate::domain::common_lisp::common_lisp_symbol_name_eq;
use crate::domain::definition::DefinitionCategory;
use crate::domain::sexpr::SymbolName;

use super::types::{CallGraphEdge, CallGraphNode};

pub fn insert_call_graph_node(
    nodes_by_name: &mut BTreeMap<String, CallGraphNode>,
    name: Option<&str>,
    category: DefinitionCategory,
) {
    if let Some(name) = name {
        if let Some((_, node)) = nodes_by_name
            .iter_mut()
            .find(|(existing, _)| common_lisp_symbol_name_eq(existing, name))
        {
            node.definition_count += 1;
            node.categories.insert(category);
        } else {
            nodes_by_name.insert(
                name.to_string(),
                CallGraphNode {
                    name: name.to_string(),
                    definition_count: 1,
                    categories: BTreeSet::from([category]),
                },
            );
        }
    }
}

pub fn build_call_graph_edge(
    call: CallReportItem,
    nodes_by_name: &BTreeMap<String, CallGraphNode>,
) -> CallGraphEdge {
    let categories = nodes_by_name
        .iter()
        .find(|(name, _)| common_lisp_symbol_name_eq(name, &call.head))
        .map(|(_, node)| node)
        .map(|node| node.categories.clone())
        .unwrap_or_default();

    CallGraphEdge {
        caller: call.enclosing_definition,
        callee: call.head,
        path: call.path,
        argument_count: call.argument_count,
        span: call.span,
        internal: !categories.is_empty(),
        callee_categories: categories,
    }
}

pub fn call_graph_edge_matches(edge: &CallGraphEdge, symbol: Option<&SymbolName>) -> bool {
    symbol
        .map(|symbol| {
            edge.caller
                .as_deref()
                .is_some_and(|caller| common_lisp_symbol_name_eq(caller, symbol.as_str()))
                || common_lisp_symbol_name_eq(&edge.callee, symbol.as_str())
        })
        .unwrap_or(true)
}
