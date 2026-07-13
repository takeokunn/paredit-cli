use std::collections::BTreeMap;
use std::str::FromStr;

use anyhow::Result;

use crate::domain::call_graph_report::{
    CallGraphNode, CallGraphNodeIndex, build_call_graph_edge, call_graph_edge_matches,
    insert_call_graph_node,
};
use crate::domain::call_report::build_call_report;
use crate::domain::common_lisp::{
    common_lisp_symbol_reference_eq, common_lisp_symbol_reference_needle,
};
use crate::domain::sexpr::SymbolName;
use crate::domain::signature_report::{SignatureCallItem, classify_signature_call};

mod definitions;
mod references;
mod summary;
mod syntax;
mod types;

use definitions::{collect_impact_definitions, impact_definition_matches_signature};
use references::{count_non_call_references, matching_symbol_occurrences};

pub use summary::{
    impact_risks, impact_status_counts, raw_refactor_risks, summarize_impact_reports,
};
pub use types::{
    ImpactDefinitionItem, ImpactReportFile, ImpactReportSource, ImpactSymbolOccurrence,
    ImpactSymbolOccurrenceContext,
};

use crate::domain::refactor_plan::RefactorPlanSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactRiskLevel {
    Info,
    Warning,
    Error,
}

impl ImpactRiskLevel {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl FromStr for ImpactRiskLevel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "info" => Ok(Self::Info),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            _ => Err(format!("unknown impact risk level: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImpactReportPolicyOptions {
    fail_on_risk_level: Option<ImpactRiskLevel>,
    require_definitions: Option<usize>,
    require_references: Option<usize>,
    require_calls: Option<usize>,
}

impl ImpactReportPolicyOptions {
    pub fn new(
        fail_on_risk_level: Option<ImpactRiskLevel>,
        require_definitions: Option<usize>,
        require_references: Option<usize>,
        require_calls: Option<usize>,
    ) -> Result<Self, String> {
        Self::validate_threshold("require-definitions", require_definitions)?;
        Self::validate_threshold("require-references", require_references)?;
        Self::validate_threshold("require-calls", require_calls)?;

        Ok(Self {
            fail_on_risk_level,
            require_definitions,
            require_references,
            require_calls,
        })
    }

    fn validate_threshold(label: &str, value: Option<usize>) -> Result<(), String> {
        if matches!(value, Some(0)) {
            return Err(format!("{label} must be greater than zero"));
        }
        Ok(())
    }

    pub const fn fail_on_risk_level(self) -> Option<ImpactRiskLevel> {
        self.fail_on_risk_level
    }

    pub const fn require_definitions(self) -> Option<usize> {
        self.require_definitions
    }

    pub const fn require_references(self) -> Option<usize> {
        self.require_references
    }

    pub const fn require_calls(self) -> Option<usize> {
        self.require_calls
    }
}

#[derive(Debug)]
pub struct ImpactRisk {
    pub level: ImpactRiskLevel,
    pub code: &'static str,
    pub message: String,
    pub count: usize,
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

pub fn evaluate_impact_report_policy(
    options: ImpactReportPolicyOptions,
    summary: &RefactorPlanSummary,
    risk_level: ImpactRiskLevel,
) -> ImpactReportPolicy {
    let mut violations = Vec::new();

    if let Some(threshold) = options.fail_on_risk_level() {
        if risk_level >= threshold {
            violations.push(format!(
                "--fail-on-risk-level {} failed with {} risk",
                threshold.label(),
                risk_level.label()
            ));
        }
    }
    if let Some(required) = options.require_definitions() {
        if summary.definition_count < required {
            violations.push(format!(
                "--require-definitions expected at least {required}, found {}",
                summary.definition_count
            ));
        }
    }
    if let Some(required) = options.require_references() {
        if summary.reference_count < required {
            violations.push(format!(
                "--require-references expected at least {required}, found {}",
                summary.reference_count
            ));
        }
    }
    if let Some(required) = options.require_calls() {
        if summary.call_count < required {
            violations.push(format!(
                "--require-calls expected at least {required}, found {}",
                summary.call_count
            ));
        }
    }

    ImpactReportPolicy {
        fail_on_risk_level: options.fail_on_risk_level(),
        require_definitions: options.require_definitions(),
        require_references: options.require_references(),
        require_calls: options.require_calls(),
        definition_count: summary.definition_count,
        reference_count: summary.reference_count,
        call_count: summary.call_count,
        inbound_edge_count: summary.inbound_edge_count,
        non_call_reference_count: summary.non_call_reference_count,
        signature_mismatch_count: summary.signature_mismatch_count,
        risk_level,
        passed: violations.is_empty(),
        violations,
    }
}

pub fn build_impact_reports(
    sources: Vec<ImpactReportSource>,
    symbol: &SymbolName,
) -> Result<Vec<ImpactReportFile>> {
    let mut parsed = Vec::with_capacity(sources.len());
    let mut nodes_by_name = BTreeMap::<String, CallGraphNode>::new();
    let mut node_index = CallGraphNodeIndex::new();
    let mut definitions_by_name = BTreeMap::<String, Vec<(usize, Option<usize>)>>::new();

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
                    .filter(|entry| entry.span.contains_span(occurrence.span))
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
                &mut node_index,
                definition.name.as_deref(),
                definition.category,
            );

            if impact_definition_matches_signature(definition, None) {
                if let (Some(name), Some(arity)) = (&definition.name, definition.parameter_arity) {
                    definitions_by_name
                        .entry(common_lisp_symbol_reference_needle(name))
                        .or_default()
                        .push(arity);
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
                        let (expected_parameter_arity, status) =
                            classify_signature_call(&definitions_by_name, &call);
                        SignatureCallItem {
                            call,
                            expected_parameter_arity,
                            status,
                        }
                    })
                    .collect::<Vec<_>>();
                let edges = all_calls
                    .into_iter()
                    .map(|call| build_call_graph_edge(call, &nodes_by_name, &node_index))
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

                ImpactReportFile::new(
                    path,
                    dialect,
                    package,
                    definitions,
                    references,
                    calls,
                    inbound_edges,
                    outbound_edges,
                    non_call_reference_count,
                )
            },
        )
        .collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_stable() {
        assert_eq!(ImpactRiskLevel::Info.label(), "info");
        assert_eq!(ImpactRiskLevel::Warning.label(), "warning");
        assert_eq!(ImpactRiskLevel::Error.label(), "error");
    }

    #[test]
    fn validates_thresholds() {
        assert!(ImpactReportPolicyOptions::new(None, Some(1), Some(2), Some(3)).is_ok());
        assert_eq!(
            ImpactReportPolicyOptions::new(None, Some(0), None, None).unwrap_err(),
            "require-definitions must be greater than zero"
        );
        assert_eq!(
            ImpactReportPolicyOptions::new(None, None, Some(0), None).unwrap_err(),
            "require-references must be greater than zero"
        );
        assert_eq!(
            ImpactReportPolicyOptions::new(None, None, None, Some(0)).unwrap_err(),
            "require-calls must be greater than zero"
        );
    }

    #[test]
    fn evaluates_policy_failures() {
        let summary = RefactorPlanSummary {
            file_count: 1,
            definition_count: 0,
            reference_count: 1,
            call_count: 0,
            inbound_edge_count: 0,
            outbound_edge_count: 0,
            non_call_reference_count: 1,
            signature_mismatch_count: 0,
            safe_to_automate: false,
        };

        let policy = evaluate_impact_report_policy(
            ImpactReportPolicyOptions::new(
                Some(ImpactRiskLevel::Warning),
                Some(1),
                Some(2),
                Some(1),
            )
            .unwrap(),
            &summary,
            ImpactRiskLevel::Error,
        );

        assert!(!policy.passed);
        assert_eq!(policy.violations.len(), 4);
    }
}
