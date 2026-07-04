use super::*;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path};
use proptest::prelude::*;

fn target(input: &str) -> ExpressionView {
    let tree = SyntaxTree::parse(input).expect("parse fixture");
    tree.select_path(&"0".parse::<Path>().expect("path"))
        .expect("select fixture")
        .view()
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
    let plan = plan_unthread_expression(UnthreadExpressionRequest {
        input,
        dialect: Dialect::Clojure,
        path: Some("0".parse().expect("path")),
        target: target(input),
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
    let plan = plan_unthread_expression(UnthreadExpressionRequest {
        input,
        dialect: Dialect::Clojure,
        path: Some("0".parse().expect("path")),
        target: target(input),
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
        let plan = plan_unthread_expression(UnthreadExpressionRequest {
            input: &input,
            dialect: Dialect::Clojure,
            path: Some("0".parse().expect("path")),
            target: target(&input),
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
