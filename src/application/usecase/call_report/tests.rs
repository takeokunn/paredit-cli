use proptest::prelude::*;

use crate::application::usecase::call_report::build_call_report;
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{SymbolName, SyntaxTree};

fn parse(input: &str) -> SyntaxTree {
    SyntaxTree::parse(input).expect("test input should parse")
}

#[test]
fn reports_calls_without_definition_forms_by_default() {
    let tree = parse("(defun f (x) (g x) (h))\n(g 1)");
    let calls = build_call_report(&tree, Dialect::CommonLisp, None, false).unwrap();

    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].head, "g");
    assert_eq!(calls[0].argument_count, 1);
    assert_eq!(calls[0].enclosing_definition.as_deref(), Some("f"));
    assert_eq!(calls[1].head, "h");
    assert_eq!(calls[1].argument_count, 0);
    assert_eq!(calls[1].enclosing_definition.as_deref(), Some("f"));
    assert_eq!(calls[2].head, "g");
    assert_eq!(calls[2].enclosing_definition, None);
}

#[test]
fn can_include_definition_forms_for_inventory_reports() {
    let tree = parse("(defun f (x) (g x))");
    let calls = build_call_report(&tree, Dialect::CommonLisp, None, true).unwrap();

    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].head, "defun");
    assert_eq!(calls[0].category, Some(DefinitionCategory::Function));
    assert_eq!(calls[1].head, "g");
    assert_eq!(calls[1].category, None);
}

#[test]
fn filters_by_symbol() {
    let tree = parse("(defun f (x) (g x) (h x) (g 1 2))");
    let symbol = SymbolName::new("g").unwrap();
    let calls = build_call_report(&tree, Dialect::CommonLisp, Some(&symbol), false).unwrap();

    assert_eq!(calls.len(), 2);
    assert!(calls.iter().all(|call| call.head == "g"));
    assert_eq!(calls[0].argument_count, 1);
    assert_eq!(calls[1].argument_count, 2);
}

#[test]
fn skips_common_lisp_flet_local_callable_calls() {
    let tree = parse("(defun main () (flet ((helper (x) (target x))) (helper 1) (target 2)))");
    let calls = build_call_report(&tree, Dialect::CommonLisp, None, false).unwrap();
    let heads = calls
        .iter()
        .map(|call| call.head.as_str())
        .collect::<Vec<_>>();

    assert_eq!(heads, vec!["target", "target"]);
}

#[test]
fn skips_common_lisp_labels_local_callable_calls_in_definition_bodies() {
    let tree = parse("(defun main () (labels ((helper (x) (helper x) (target x))) (helper 1)))");
    let calls = build_call_report(&tree, Dialect::CommonLisp, None, false).unwrap();
    let heads = calls
        .iter()
        .map(|call| call.head.as_str())
        .collect::<Vec<_>>();

    assert_eq!(heads, vec!["target"]);
}

#[test]
fn skips_common_lisp_macrolet_local_macro_calls() {
    let tree = parse("(defun main () (macrolet ((helper (x) (list 'target x))) (helper 1)))");
    let symbol = SymbolName::new("helper").unwrap();
    let calls = build_call_report(&tree, Dialect::CommonLisp, Some(&symbol), false).unwrap();

    assert!(calls.is_empty());
}

#[test]
fn skips_common_lisp_compiler_macrolet_local_macro_calls() {
    let tree =
        parse("(defun main () (compiler-macrolet ((helper (x) (list 'target x))) (helper 1)))");
    let symbol = SymbolName::new("helper").unwrap();
    let calls = build_call_report(&tree, Dialect::CommonLisp, Some(&symbol), false).unwrap();

    assert!(calls.is_empty());
}

#[test]
fn skips_common_lisp_defmethod_specialized_lambda_list_calls() {
    let tree = parse("(defmethod render :around ((node widget) stream) (draw node stream))");
    let calls = build_call_report(&tree, Dialect::CommonLisp, None, false).unwrap();

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].head, "draw");
    assert_eq!(calls[0].enclosing_definition.as_deref(), Some("render"));
}

fn symbol_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,8}".prop_filter("exclude definition heads", |symbol| {
        !matches!(
            symbol.as_str(),
            "defun" | "fn" | "lambda" | "let" | "nil" | "t" | "true" | "false"
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_reports_generated_call_heads_and_argument_counts(
        function in symbol_strategy(),
        callee in symbol_strategy(),
        argument_count in 0usize..6,
    ) {
        prop_assume!(function != callee);
        let args = (0..argument_count)
            .map(|index| format!("x{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let input = format!("(defun {function} () ({callee}{prefix}{args}))", prefix = if args.is_empty() { "" } else { " " });
        let tree = SyntaxTree::parse(&input).unwrap();
        let symbol = SymbolName::new(callee.clone()).unwrap();
        let calls = build_call_report(&tree, Dialect::CommonLisp, Some(&symbol), false).unwrap();

        prop_assert_eq!(calls.len(), 1);
        prop_assert_eq!(calls[0].head.as_str(), callee.as_str());
        prop_assert_eq!(calls[0].argument_count, argument_count);
        prop_assert_eq!(calls[0].enclosing_definition.as_deref(), Some(function.as_str()));
    }
}
