use std::collections::BTreeMap;

use anyhow::Result;

use crate::application::usecase::call_graph_report::{
    CallGraphNode, build_call_graph_edge, call_graph_edge_matches, insert_call_graph_node,
};
use crate::application::usecase::call_report::build_call_report;
use crate::application::usecase::signature_report::{SignatureCallItem, classify_signature_call};
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::sexpr::SymbolName;

mod definitions;
mod policy;
mod references;
mod summary;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

use definitions::{collect_impact_definitions, impact_definition_matches_signature};
use references::{count_non_call_references, matching_symbol_occurrences, span_contains};

pub use policy::evaluate_impact_report_policy;
pub use summary::{
    impact_risks, impact_status_counts, raw_refactor_risks, summarize_impact_reports,
};
pub use types::{
    ImpactDefinitionItem, ImpactReportFile, ImpactReportPolicy, ImpactReportPolicyOptions,
    ImpactReportSource, ImpactRisk, ImpactRiskLevel, ImpactSymbolOccurrence,
    ImpactSymbolOccurrenceContext,
};

pub fn build_impact_reports(
    sources: Vec<ImpactReportSource>,
    symbol: &SymbolName,
) -> Result<Vec<ImpactReportFile>> {
    let mut parsed = Vec::with_capacity(sources.len());
    let mut nodes_by_name = BTreeMap::<String, CallGraphNode>::new();
    let mut definitions_by_name = BTreeMap::<String, Vec<usize>>::new();

    for source in sources {
        let outline = source
            .tree
            .outline(|head| source.dialect.is_definition_head(head));
        let (package, all_definitions) = collect_impact_definitions(&source.tree, source.dialect)?;
        let references = matching_symbol_occurrences(source.dialect, &source.tree, symbol)
            .into_iter()
            .map(|occurrence| ImpactSymbolOccurrence {
                path: occurrence.path.to_string(),
                span: occurrence.span,
                context: outline
                    .iter()
                    .filter(|entry| span_contains(entry.span, occurrence.span))
                    .min_by_key(|entry| entry.span.end().get() - entry.span.start().get())
                    .map(|entry| ImpactSymbolOccurrenceContext {
                        path: entry.path.to_string(),
                        span: entry.span,
                        head: entry.head.clone(),
                        definition_like: entry.definition_like,
                    }),
            })
            .collect::<Vec<_>>();
        let definitions = all_definitions
            .iter()
            .filter(|definition| {
                definition
                    .name
                    .as_deref()
                    .is_some_and(|name| common_lisp_symbol_reference_eq(name, symbol.as_str()))
            })
            .cloned()
            .collect::<Vec<_>>();
        let calls = build_call_report(&source.tree, source.dialect, Some(symbol), false)?;
        let all_calls = build_call_report(&source.tree, source.dialect, None, false)?;

        for definition in &all_definitions {
            insert_call_graph_node(
                &mut nodes_by_name,
                definition.name.as_deref(),
                definition.category,
            );

            if impact_definition_matches_signature(definition, None) {
                if let (Some(name), Some(parameter_count)) =
                    (&definition.name, definition.parameter_count)
                {
                    definitions_by_name
                        .entry(name.clone())
                        .or_default()
                        .push(parameter_count);
                }
            }
        }

        parsed.push((
            source.path,
            source.dialect,
            package,
            definitions,
            references,
            calls,
            all_calls,
        ));
    }

    Ok(parsed
        .into_iter()
        .map(
            |(path, dialect, package, definitions, references, calls, all_calls)| {
                let calls = calls
                    .into_iter()
                    .map(|call| {
                        let (expected_parameter_count, status) =
                            classify_signature_call(&definitions_by_name, &call);
                        SignatureCallItem {
                            call,
                            expected_parameter_count,
                            status,
                        }
                    })
                    .collect::<Vec<_>>();
                let edges = all_calls
                    .into_iter()
                    .map(|call| build_call_graph_edge(call, &nodes_by_name))
                    .filter(|edge| call_graph_edge_matches(edge, Some(symbol)))
                    .collect::<Vec<_>>();
                let inbound_edges = edges
                    .iter()
                    .filter(|edge| common_lisp_symbol_reference_eq(&edge.callee, symbol.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                let outbound_edges = edges
                    .iter()
                    .filter(|edge| {
                        edge.caller.as_deref().is_some_and(|caller| {
                            common_lisp_symbol_reference_eq(caller, symbol.as_str())
                        })
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                let non_call_reference_count =
                    count_non_call_references(&path, &references, &definitions, &calls);

                ImpactReportFile {
                    path,
                    dialect,
                    package,
                    definitions,
                    references,
                    calls,
                    inbound_edges,
                    outbound_edges,
                    non_call_reference_count,
                }
            },
        )
        .collect::<Vec<_>>())
}
