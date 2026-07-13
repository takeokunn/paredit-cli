use super::*;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path, SymbolName};
use proptest::prelude::*;

fn parsed(input: &str) -> (SyntaxTree, ExpressionView) {
    let tree = SyntaxTree::parse(input).expect("parse fixture");
    let target = tree
        .select_path(&"0".parse::<Path>().expect("path"))
        .expect("select fixture")
        .view();
    (tree, target)
}

fn symbol_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,8}".prop_filter("reserved symbol", |name| {
        !matches!(
            name.as_str(),
            "defun" | "false" | "fn" | "lambda" | "let" | "nil" | "t" | "true"
        )
    })
}

#[test]
fn plans_unthread_first_pipeline_into_nested_call() {
    let input = "(-> value (normalize mode) render)";
    let (tree, target) = parsed(input);
    let plan = plan_unthread_expression(UnthreadExpressionRequest {
        input,
        tree: &tree,
        dialect: Dialect::Clojure,
        path: Some("0".parse().expect("path")),
        target,
        style: None,
        operator: None,
    })
    .expect("plan");

    assert_eq!(plan.style, UnthreadStyle::First);
    assert_eq!(plan.base, "value");
    assert_eq!(plan.replacement, "(render (normalize value mode))");
    assert_eq!(plan.steps.len(), 2);
    assert!(plan.changed);
}

#[test]
fn plans_unthread_last_pipeline_into_nested_call() {
    let input = "(->> rows (filter pred) (map f))";
    let (tree, target) = parsed(input);
    let plan = plan_unthread_expression(UnthreadExpressionRequest {
        input,
        tree: &tree,
        dialect: Dialect::Clojure,
        path: Some("0".parse().expect("path")),
        target,
        style: None,
        operator: None,
    })
    .expect("plan");

    assert_eq!(plan.style, UnthreadStyle::Last);
    assert_eq!(plan.base, "rows");
    assert_eq!(plan.replacement, "(map f (filter pred rows))");
    assert_eq!(plan.steps.len(), 2);
    assert!(plan.changed);
}

#[test]
fn rejects_style_alone_on_an_unrecognized_operator() {
    let input = "(+ a b)";
    let (tree, target) = parsed(input);
    let err = plan_unthread_expression(UnthreadExpressionRequest {
        input,
        tree: &tree,
        dialect: Dialect::CommonLisp,
        path: Some("0".parse().expect("path")),
        target,
        style: Some(UnthreadStyle::First),
        operator: None,
    })
    .expect_err("bare --style must not accept an unrecognized operator");

    assert!(err.to_string().contains("--operator"));
}

#[test]
fn accepts_style_with_explicit_operator_confirming_a_custom_pipeline() {
    let input = "(my-pipe value (normalize mode) render)";
    let (tree, target) = parsed(input);
    let plan = plan_unthread_expression(UnthreadExpressionRequest {
        input,
        tree: &tree,
        dialect: Dialect::CommonLisp,
        path: Some("0".parse().expect("path")),
        target,
        style: Some(UnthreadStyle::First),
        operator: Some(SymbolName::new("my-pipe").expect("symbol")),
    })
    .expect("plan");

    assert_eq!(plan.replacement, "(render (normalize value mode))");
}

#[test]
fn rejects_pipeline_with_an_interior_comment() {
    let input = "(-> value\n    ;; note\n    (normalize mode)\n    render)";
    let (tree, target) = parsed(input);
    let err = plan_unthread_expression(UnthreadExpressionRequest {
        input,
        tree: &tree,
        dialect: Dialect::Clojure,
        path: Some("0".parse().expect("path")),
        target,
        style: None,
        operator: None,
    })
    .expect_err("a comment inside the pipeline must not be silently discarded");

    assert!(err.to_string().contains("comment"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_unthread_first_output_is_parseable_and_stable(
        base in symbol_strategy(),
        inner in symbol_strategy(),
        outer in symbol_strategy(),
        arg in symbol_strategy(),
    ) {
        prop_assume!(base != inner);
        prop_assume!(base != outer);
        prop_assume!(inner != outer);

        let input = format!("(-> {base} ({inner} {arg}) {outer})");
        let (tree, target) = parsed(&input);
        let plan = plan_unthread_expression(UnthreadExpressionRequest {
            input: &input,
            tree: &tree,
            dialect: Dialect::Clojure,
            path: Some("0".parse().expect("path")),
            target,
            style: None,
            operator: None,
        })
        .expect("plan");

        prop_assert_eq!(
            plan.replacement,
            format!("({outer} ({inner} {base} {arg}))")
        );
        prop_assert!(SyntaxTree::parse(&plan.rewritten).is_ok());
        prop_assert!(plan.changed);
    }
}
