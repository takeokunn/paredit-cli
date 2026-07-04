use proptest::{prelude::*, test_runner::TestCaseError};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path, SymbolName, SyntaxTree};

use super::{RemoveUnusedBindingRequest, plan_remove_unused_binding};

fn target(input: &str) -> ExpressionView {
    let tree = SyntaxTree::parse(input).expect("parse");
    tree.select_path(&"0".parse::<Path>().expect("path"))
        .expect("select")
        .view()
}

#[test]
fn plans_common_lisp_single_unused_binding() {
    let input = "(let ((unused 1) (used 2)) used)";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: Some("0".parse().expect("path")),
        target: target(input),
        name: Some(&SymbolName::new("unused").expect("symbol")),
        all_bindings: false,
        allow_drop_value: false,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("unused"));
    assert_eq!(plan.binding_value.as_deref(), Some("1"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(plan.replacement, "(let ( (used 2)) used)");
    assert_eq!(plan.rewritten, "(let ( (used 2)) used)");
    assert!(plan.dropped_value_requires_review);
    assert!(plan.changed);
}

#[test]
fn rejects_referenced_binding() {
    let input = "(let ((x 1)) x)";
    let error = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("x").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect_err("referenced binding");

    assert!(error.to_string().contains("zero in-scope references"));
}

#[test]
fn plans_unused_binding_ignoring_shadowed_lambda_parameter() {
    let input = "(let ((x 1) (used 2)) (list used (lambda (x) x)))";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("x").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("x"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(
        plan.replacement,
        "(let ( (used 2)) (list used (lambda (x) x)))"
    );
}

#[test]
fn rejects_reference_before_shadowed_lambda_parameter() {
    let input = "(let ((x 1)) (list x (lambda (x) x)))";
    let error = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("x").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect_err("outer reference");

    assert!(error.to_string().contains("zero in-scope references"));
}

#[test]
fn plans_unused_binding_ignoring_shadowed_inner_let() {
    let input = "(let ((x 1) (used 2)) (let ((x 3)) x) used)";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("x").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("x"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(plan.replacement, "(let ( (used 2)) (let ((x 3)) x) used)");
}

#[test]
fn plans_all_unused_bindings_by_replacing_form_with_body() {
    let input = "(let ((unused 1)) body)";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: None,
        all_bindings: true,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.bindings.len(), 1);
    assert_eq!(plan.replacement, "body");
    assert_eq!(plan.rewritten, "body");
}

#[test]
fn plans_clojure_vector_binding() {
    let input = "(let [unused 1 used 2] used)";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::Clojure,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("unused").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("unused"));
    assert_eq!(plan.replacement, "(let [ used 2] used)");
    assert_eq!(plan.rewritten, "(let [ used 2] used)");
}

#[test]
fn plans_clojure_vector_unused_binding_ignoring_shadowed_fn_parameter() {
    let input = "(let [x 1 used 2] (list used (fn [x] x)))";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::Clojure,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("x").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("x"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(plan.replacement, "(let [ used 2] (list used (fn [x] x)))");
}

proptest! {
    #[test]
    fn pbt_unused_single_binding_output_remains_parseable(
        name in "[a-z][a-z0-9]{0,8}",
        value in "[-]?[0-9]{1,4}",
        body in "[a-z][a-z0-9]{0,8}",
    ) {
        prop_assume!(body != name);
        let input = format!("(let (({name} {value})) {body})");
        let symbol = SymbolName::new(name.clone())
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            path: None,
            target: target(&input),
            name: Some(&symbol),
            all_bindings: false,
            allow_drop_value: true,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(plan.reference_count, Some(0));
        prop_assert_eq!(&plan.rewritten, &body);
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }

    #[test]
    fn pbt_referenced_single_binding_is_rejected(
        name in "[a-z][a-z0-9]{0,8}",
        value in "[-]?[0-9]{1,4}",
    ) {
        let input = format!("(let (({name} {value})) {name})");
        let symbol = SymbolName::new(name.clone())
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let result = plan_remove_unused_binding(RemoveUnusedBindingRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            path: None,
            target: target(&input),
            name: Some(&symbol),
            all_bindings: false,
            allow_drop_value: true,
        });
        prop_assert!(result.is_err());
    }

    #[test]
    fn pbt_shadowed_lambda_parameter_does_not_keep_outer_binding(
        name in "[a-z][a-z0-9]{0,8}",
        other in "[a-z][a-z0-9]{0,8}",
        value in "[-]?[0-9]{1,4}",
    ) {
        prop_assume!(other != name);
        let input = format!(
            "(let (({name} {value}) ({other} 2)) (list {other} (lambda ({name}) {name})))"
        );
        let symbol = SymbolName::new(name.clone())
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            path: None,
            target: target(&input),
            name: Some(&symbol),
            all_bindings: false,
            allow_drop_value: true,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(plan.reference_count, Some(0));
        prop_assert!(plan.changed);
        let expected_lambda = format!("(lambda ({}) {})", name, name);
        prop_assert!(plan.rewritten.contains(&expected_lambda));
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }
}
