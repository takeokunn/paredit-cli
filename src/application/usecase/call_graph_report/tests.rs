use std::path::PathBuf;

use proptest::prelude::*;

use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{SymbolName, SyntaxTree};

use super::*;

fn parse(input: &str) -> SyntaxTree {
    SyntaxTree::parse(input).expect("valid lisp")
}

fn source(input: &str) -> CallGraphReportSource {
    CallGraphReportSource {
        path: PathBuf::from("sample.lisp"),
        dialect: Dialect::CommonLisp,
        tree: parse(input),
    }
}

#[test]
fn builds_internal_edges_across_definitions() {
    let report = build_call_graph_report(
        vec![source("(defun helper (x) x)\n(defun main (y) (helper y))")],
        false,
        None,
    )
    .unwrap();

    assert_eq!(report.nodes_by_name.len(), 2);
    assert!(report.nodes_by_name.contains_key("helper"));
    assert!(report.nodes_by_name.contains_key("main"));
    assert_eq!(report.files[0].definitions.len(), 2);
    assert_eq!(report.files[0].edges.len(), 1);
    assert_eq!(report.files[0].edges[0].caller.as_deref(), Some("main"));
    assert_eq!(report.files[0].edges[0].callee, "helper");
    assert!(report.files[0].edges[0].internal);
    assert!(
        report.files[0].edges[0]
            .callee_categories
            .contains(&DefinitionCategory::Function)
    );
}

#[test]
fn can_include_external_edges_and_filter_by_symbol() {
    let symbol = SymbolName::new("missing").unwrap();
    let report = build_call_graph_report(
        vec![source(
            "(defun main (y) (missing y) (helper y))\n(defun helper (x) x)",
        )],
        true,
        Some(&symbol),
    )
    .unwrap();

    assert_eq!(report.files[0].edges.len(), 1);
    assert_eq!(report.files[0].edges[0].caller.as_deref(), Some("main"));
    assert_eq!(report.files[0].edges[0].callee, "missing");
    assert!(!report.files[0].edges[0].internal);
}

#[test]
fn skips_common_lisp_local_callable_edges_to_shadowed_global_definitions() {
    let report = build_call_graph_report(
        vec![source(
            "(defun helper (x y) y)\n(defun main () (flet ((helper (x) (target x))) (helper 1)))\n(defun target (x) x)",
        )],
        false,
        None,
    )
    .unwrap();
    let edges = &report.files[0].edges;

    assert!(
        !edges
            .iter()
            .any(|edge| edge.caller.as_deref() == Some("main") && edge.callee == "helper")
    );
    assert!(
        edges
            .iter()
            .any(|edge| edge.caller.as_deref() == Some("main") && edge.callee == "target")
    );
}

#[test]
fn counts_common_lisp_macro_and_method_lambda_lists() {
    let report = build_call_graph_report(
        vec![source(
            "(define-compiler-macro fast-add (x y) (helper x y))\n(defmethod render :around ((node widget) stream) (draw node stream))\n(defun helper (x y) y)\n(defun draw (node stream) stream)",
        )],
        false,
        None,
    )
    .unwrap();

    let definitions = &report.files[0].definitions;
    let fast_add = definitions
        .iter()
        .find(|definition| definition.name.as_deref() == Some("fast-add"))
        .expect("compiler macro definition");
    let render = definitions
        .iter()
        .find(|definition| definition.name.as_deref() == Some("render"))
        .expect("method definition");

    assert_eq!(fast_add.parameter_count, 2);
    assert_eq!(render.parameter_count, 2);
    assert!(
        !report.files[0]
            .edges
            .iter()
            .any(|edge| edge.caller.as_deref() == Some("render") && edge.callee == "node")
    );
    assert!(
        report.files[0]
            .edges
            .iter()
            .any(|edge| edge.caller.as_deref() == Some("render") && edge.callee == "draw")
    );
}

#[test]
fn policy_counts_inbound_edges_without_self_recursive_calls() {
    let symbol = SymbolName::new("target").unwrap();
    let report = build_call_graph_report(
        vec![source(
            "(defun target (x) (target x))\n(defun caller (y) (target y))",
        )],
        false,
        Some(&symbol),
    )
    .unwrap();

    let policy = evaluate_call_graph_policy(&report.files, Some(&symbol), true, Some(2), Some(2));

    assert_eq!(policy.edge_count, 2);
    assert_eq!(policy.internal_edge_count, 2);
    assert_eq!(
        policy.inbound_callers.iter().cloned().collect::<Vec<_>>(),
        vec!["caller".to_string()]
    );
}

fn symbol_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,8}".prop_map(|symbol| symbol)
}

proptest! {
    #[test]
    fn pbt_internal_edges_preserve_generated_callee_and_arity(
        caller in symbol_strategy(),
        callee in symbol_strategy(),
        arity in 0usize..6,
    ) {
        prop_assume!(caller != callee);
        let params = (0..arity)
            .map(|index| format!("x{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let args = (0..arity)
            .map(|index| format!("x{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let input = format!(
            "(defun {callee} ({params}) {callee})\n(defun {caller} ({params}) ({callee} {args}))"
        );

        let report = build_call_graph_report(vec![source(&input)], false, None).unwrap();
        let matching_edges = report.files[0]
            .edges
            .iter()
            .filter(|edge| edge.caller.as_deref() == Some(caller.as_str()) && edge.callee == callee)
            .collect::<Vec<_>>();

        prop_assert_eq!(matching_edges.len(), 1);
        prop_assert_eq!(matching_edges[0].argument_count, arity);
        prop_assert!(matching_edges[0].internal);
    }
}
