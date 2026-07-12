use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::application::usecase::call_report::CallReportItem;
use crate::domain::common_lisp::{
    common_lisp_symbol_reference_eq, common_lisp_symbol_reference_needle,
};
use crate::domain::definition::DefinitionCategory;
use crate::domain::sexpr::SymbolName;

use super::types::{CallGraphEdge, CallGraphNode};

/// Maps each node's `common_lisp_symbol_reference_needle` to its key in
/// `nodes_by_name`, so node lookups cost one hash probe instead of a linear
/// scan with per-entry qualifier stripping. `insert_call_graph_node` keeps at
/// most one `nodes_by_name` entry per needle equivalence class, which makes
/// the index lookup equivalent to the linear "first match" search it
/// replaces.
pub type CallGraphNodeIndex = HashMap<String, String>;

pub fn insert_call_graph_node(
    nodes_by_name: &mut BTreeMap<String, CallGraphNode>,
    node_index: &mut CallGraphNodeIndex,
    name: Option<&str>,
    category: DefinitionCategory,
) {
    if let Some(name) = name {
        let needle = common_lisp_symbol_reference_needle(name);
        if let Some(node) = node_index
            .get(&needle)
            .and_then(|key| nodes_by_name.get_mut(key))
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
            node_index.insert(needle, name.to_string());
        }
    }
}

pub fn build_call_graph_edge(
    call: CallReportItem,
    nodes_by_name: &BTreeMap<String, CallGraphNode>,
    node_index: &CallGraphNodeIndex,
) -> CallGraphEdge {
    let categories = node_index
        .get(&common_lisp_symbol_reference_needle(&call.head))
        .and_then(|key| nodes_by_name.get(key))
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
                .is_some_and(|caller| common_lisp_symbol_reference_eq(caller, symbol.as_str()))
                || common_lisp_symbol_reference_eq(&edge.callee, symbol.as_str())
        })
        .unwrap_or(true)
}
