use std::collections::{BTreeMap, BTreeSet};

use crate::application::usecase::call_report::CallReportItem;
use crate::domain::definition::DefinitionCategory;
use crate::domain::sexpr::SymbolName;

use super::types::{CallGraphEdge, CallGraphNode};

pub fn insert_call_graph_node(
    nodes_by_name: &mut BTreeMap<String, CallGraphNode>,
    name: Option<&str>,
    category: DefinitionCategory,
) {
    if let Some(name) = name {
        nodes_by_name
            .entry(name.to_string())
            .and_modify(|node| {
                node.definition_count += 1;
                node.categories.insert(category);
            })
            .or_insert_with(|| CallGraphNode {
                name: name.to_string(),
                definition_count: 1,
                categories: BTreeSet::from([category]),
            });
    }
}

pub fn build_call_graph_edge(
    call: CallReportItem,
    nodes_by_name: &BTreeMap<String, CallGraphNode>,
) -> CallGraphEdge {
    let categories = nodes_by_name
        .get(&call.head)
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
            edge.caller.as_deref() == Some(symbol.as_str()) || edge.callee == symbol.as_str()
        })
        .unwrap_or(true)
}
