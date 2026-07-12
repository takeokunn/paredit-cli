use std::collections::BTreeMap;

use anyhow::Result;

use crate::application::usecase::call_report::build_call_report;
use crate::domain::sexpr::SymbolName;

mod definitions;
mod graph;
mod policy;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

pub use graph::{
    CallGraphNodeIndex, build_call_graph_edge, call_graph_edge_matches, insert_call_graph_node,
};
pub use policy::evaluate_call_graph_policy;
pub use types::{
    CallGraphDefinitionItem, CallGraphEdge, CallGraphFile, CallGraphNode, CallGraphPolicy,
    CallGraphPolicyOptions, CallGraphReport, CallGraphReportSource,
};

pub fn build_call_graph_report(
    sources: Vec<CallGraphReportSource>,
    include_external: bool,
    symbol: Option<&SymbolName>,
) -> Result<CallGraphReport> {
    let mut parsed = Vec::with_capacity(sources.len());
    let mut nodes_by_name = BTreeMap::<String, CallGraphNode>::new();
    let mut node_index = CallGraphNodeIndex::new();

    for source in sources {
        let definitions =
            definitions::collect_call_graph_definitions(&source.tree, source.dialect)?;
        let calls = build_call_report(&source.tree, source.dialect, None, false)?;

        for definition in &definitions {
            insert_call_graph_node(
                &mut nodes_by_name,
                &mut node_index,
                definition.name.as_deref(),
                definition.category,
            );
        }

        parsed.push((source.path, source.dialect, definitions, calls));
    }

    let files = parsed
        .into_iter()
        .map(|(path, dialect, definitions, calls)| {
            let edges = calls
                .into_iter()
                .map(|call| build_call_graph_edge(call, &nodes_by_name, &node_index))
                .filter(|edge| include_external || edge.internal)
                .filter(|edge| call_graph_edge_matches(edge, symbol))
                .collect::<Vec<_>>();

            CallGraphFile {
                path,
                dialect,
                definitions,
                edges,
            }
        })
        .collect::<Vec<_>>();

    Ok(CallGraphReport {
        files,
        nodes_by_name,
    })
}
