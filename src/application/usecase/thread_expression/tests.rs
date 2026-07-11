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
fn plans_thread_first_pipeline_from_nested_calls() {
    let input = "(render (normalize value mode))";
    let (tree, target) = parsed(input);
    let plan = plan_thread_expression(ThreadExpressionRequest {
        input,
        tree: &tree,
        dialect: Dialect::Clojure,
        path: Some("0".parse().expect("path")),
        target,
        style: ThreadStyle::First,
        operator: SymbolName::new("->").expect("symbol"),
    })
    .expect("plan");

    assert_eq!(plan.base, "value");
    assert_eq!(plan.replacement, "(-> value (normalize mode) render)");
    assert_eq!(plan.steps.len(), 2);
    assert!(plan.changed);
}

#[test]
fn plans_thread_last_pipeline_from_nested_calls() {
    let input = "(map f (filter pred rows))";
    let (tree, target) = parsed(input);
    let plan = plan_thread_expression(ThreadExpressionRequest {
        input,
        tree: &tree,
        dialect: Dialect::Clojure,
        path: Some("0".parse().expect("path")),
        target,
        style: ThreadStyle::Last,
        operator: SymbolName::new("->>").expect("symbol"),
    })
    .expect("plan");

    assert_eq!(plan.base, "rows");
    assert_eq!(plan.replacement, "(->> rows (filter pred) (map f))");
    assert_eq!(plan.steps.len(), 2);
    assert!(plan.changed);
}

#[test]
fn rejects_package_qualified_already_threaded_expression() {
    let input = "(cl:-> x f)";
    let (tree, target) = parsed(input);
    let err = plan_thread_expression(ThreadExpressionRequest {
        input,
        tree: &tree,
        dialect: Dialect::CommonLisp,
        path: Some("0".parse().expect("path")),
        target,
        style: ThreadStyle::First,
        operator: SymbolName::new("->").expect("symbol"),
    })
    .expect_err("package-qualified thread operator should be rejected");

    assert!(err.to_string().contains("already threaded"));
}

#[test]
fn rejects_nested_calls_with_an_interior_comment() {
    let input = "(render\n  ;; note\n  (normalize value mode))";
    let (tree, target) = parsed(input);
    let err = plan_thread_expression(ThreadExpressionRequest {
        input,
        tree: &tree,
        dialect: Dialect::Clojure,
        path: Some("0".parse().expect("path")),
        target,
        style: ThreadStyle::First,
        operator: SymbolName::new("->").expect("symbol"),
    })
    .expect_err("a comment inside the nested calls must not be silently discarded");

    assert!(err.to_string().contains("comment"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_thread_first_output_is_parseable_and_stable(
        base in symbol_strategy(),
        inner in symbol_strategy(),
        outer in symbol_strategy(),
        arg in symbol_strategy(),
    ) {
        prop_assume!(base != inner);
        prop_assume!(base != outer);
        prop_assume!(inner != outer);

        let input = format!("({outer} ({inner} {base} {arg}))");
        let (tree, target) = parsed(&input);
        let plan = plan_thread_expression(ThreadExpressionRequest {
            input: &input,
            tree: &tree,
            dialect: Dialect::Clojure,
            path: Some("0".parse().expect("path")),
            target,
            style: ThreadStyle::First,
            operator: SymbolName::new("->").expect("symbol"),
        })
        .expect("plan");

        prop_assert_eq!(
            plan.replacement,
            format!("(-> {base} ({inner} {arg}) {outer})")
        );
        prop_assert!(SyntaxTree::parse(&plan.rewritten).is_ok());
        prop_assert!(plan.changed);
    }
}
