use proptest::{prelude::*, test_runner::TestCaseError};

use super::*;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path};

fn target(input: &str) -> ExpressionView {
    let tree = SyntaxTree::parse(input).expect("parse");
    tree.select_path(&"0".parse::<Path>().expect("path"))
        .expect("select")
        .view()
}

#[test]
fn plans_common_lisp_inline_let() {
    let input = "(let ((product (* width height))) (+ product margin))";
    let plan = plan_inline_let(InlineLetRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: Some("0".parse().expect("path")),
        target: target(input),
        allow_duplicate_evaluation: false,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_str(), "product");
    assert_eq!(plan.binding_value, "(* width height)");
    assert_eq!(plan.reference_count, 1);
    assert_eq!(plan.replacement, "(+ (* width height) margin)");
    assert_eq!(plan.rewritten, "(+ (* width height) margin)");
    assert!(plan.changed);
}

#[test]
fn rejects_duplicate_evaluation_by_default() {
    let input = "(let ((x (compute))) (+ x x))";
    let error = plan_inline_let(InlineLetRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        allow_duplicate_evaluation: false,
    })
    .expect_err("duplicate evaluation");

    assert!(error.to_string().contains("duplicate binding value"));
}

#[test]
fn plans_clojure_vector_binding() {
    let input = "(let [product (* width height)] (+ product margin))";
    let plan = plan_inline_let(InlineLetRequest {
        input,
        dialect: Dialect::Clojure,
        path: None,
        target: target(input),
        allow_duplicate_evaluation: false,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_str(), "product");
    assert_eq!(plan.rewritten, "(+ (* width height) margin)");
}

#[test]
fn plans_inline_let_without_touching_shadowed_lambda_parameter() {
    let input = "(let ((x 1)) (list x (lambda (x) x)))";
    let plan = plan_inline_let(InlineLetRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        allow_duplicate_evaluation: false,
    })
    .expect("plan");

    assert_eq!(plan.reference_count, 1);
    assert_eq!(plan.rewritten, "(list 1 (lambda (x) x))");
}

#[test]
fn rejects_only_shadowed_lambda_references_as_unused() {
    let input = "(let ((x 1)) (lambda (x) x))";
    let error = plan_inline_let(InlineLetRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        allow_duplicate_evaluation: false,
    })
    .expect_err("unused binding");

    assert!(error.to_string().contains("drop an unused binding value"));
}

#[test]
fn plans_inline_let_without_touching_shadowed_inner_let() {
    let input = "(let ((x 1)) (list x (let ((x 2)) x)))";
    let plan = plan_inline_let(InlineLetRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        allow_duplicate_evaluation: false,
    })
    .expect("plan");

    assert_eq!(plan.reference_count, 1);
    assert_eq!(plan.rewritten, "(list 1 (let ((x 2)) x))");
}

#[test]
fn plans_clojure_vector_inline_let_without_touching_shadowed_fn_parameter() {
    let input = "(let [x 1] (list x (fn [x] x)))";
    let plan = plan_inline_let(InlineLetRequest {
        input,
        dialect: Dialect::Clojure,
        path: None,
        target: target(input),
        allow_duplicate_evaluation: false,
    })
    .expect("plan");

    assert_eq!(plan.reference_count, 1);
    assert_eq!(plan.rewritten, "(list 1 (fn [x] x))");
}

proptest! {
    #[test]
    fn pbt_single_reference_inline_output_remains_parseable(
        name in "[a-z][a-z0-9]{0,8}",
        value in "[-]?[0-9]{1,4}",
        addend in "[-]?[0-9]{1,4}",
    ) {
        let input = format!("(let (({name} {value})) (+ {name} {addend}))");
        let plan = plan_inline_let(InlineLetRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            path: None,
            target: target(&input),
            allow_duplicate_evaluation: false,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(plan.reference_count, 1);
        let expected = format!("(+ {} {})", value, addend);
        prop_assert!(plan.rewritten.contains(&expected));
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }

    #[test]
    fn pbt_duplicate_evaluation_policy_controls_multi_reference_rewrites(
        name in "[a-z][a-z0-9]{0,8}",
        function in "[a-z][a-z0-9]{0,8}",
    ) {
        let input = format!("(let (({name} ({function}))) (+ {name} {name}))");
        let default_result = plan_inline_let(InlineLetRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            path: None,
            target: target(&input),
            allow_duplicate_evaluation: false,
        });
        prop_assert!(default_result.is_err());

        let plan = plan_inline_let(InlineLetRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            path: None,
            target: target(&input),
            allow_duplicate_evaluation: true,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert_eq!(plan.reference_count, 2);
        prop_assert_eq!(&plan.rewritten, &format!("(+ ({function}) ({function}))"));
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }

    #[test]
    fn pbt_shadowed_lambda_parameter_is_not_inlined(
        name in "[a-z][a-z0-9]{0,8}",
        value in "[-]?[0-9]{1,4}",
    ) {
        let input = format!("(let (({name} {value})) (list {name} (lambda ({name}) {name})))");
        let plan = plan_inline_let(InlineLetRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            path: None,
            target: target(&input),
            allow_duplicate_evaluation: false,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(plan.reference_count, 1);
        let expected_lambda = format!("(lambda ({name}) {name})");
        prop_assert!(plan.rewritten.contains(&expected_lambda));
        prop_assert_eq!(&plan.rewritten, &format!("(list {value} {expected_lambda})"));
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }
}
