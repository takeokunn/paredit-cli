use proptest::{prelude::*, test_runner::TestCaseError};

use crate::domain::sexpr::{Path, SyntaxTree};

use super::*;

fn path(value: &str) -> Path {
    value.parse().expect("path")
}

#[test]
fn plans_single_common_lisp_call() {
    let input = "(defun area (w h) (* w h))\n(print (area 3 4))";
    let plan = plan_inline_function(InlineFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        remove_definition: false,
        allow_duplicate_evaluation: false,
        allow_drop_arguments: false,
    })
    .expect("plan");

    assert_eq!(plan.function_name.as_str(), "area");
    assert_eq!(plan.calls[0].replacement, "(* 3 4)");
    assert_eq!(
        plan.rewritten,
        "(defun area (w h) (* w h))\n(print (* 3 4))"
    );
    assert!(plan.changed);
}

#[test]
fn discovers_all_calls_and_removes_definition() {
    let input = "(defun inc (x) (+ x 1))\n(print (inc 1))\n(print (inc 2))";
    let plan = plan_inline_function(InlineFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        call_paths: Vec::new(),
        all_calls: true,
        remove_definition: true,
        allow_duplicate_evaluation: false,
        allow_drop_arguments: false,
    })
    .expect("plan");

    assert_eq!(plan.calls.len(), 2);
    assert_eq!(plan.rewritten, "(print (+ 1 1))\n(print (+ 2 1))");
    assert!(plan.definition_removed);
}

#[test]
fn rejects_duplicate_evaluation_by_default() {
    let input = "(defun twice (x) (+ x x))\n(print (twice (next)))";
    let error = plan_inline_function(InlineFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        remove_definition: false,
        allow_duplicate_evaluation: false,
        allow_drop_arguments: false,
    })
    .expect_err("duplicate evaluation");

    assert!(error.to_string().contains("duplicate argument"));
}

#[test]
fn ignores_shadowed_parameter_references() {
    let input = "(defun outer (x) (let ((x 10)) x))\n(print (outer (next)))";
    let error = plan_inline_function(InlineFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        remove_definition: false,
        allow_duplicate_evaluation: false,
        allow_drop_arguments: false,
    })
    .expect_err("dropped shadowed argument");

    assert!(error.to_string().contains("drop argument"));
}

proptest! {
    #[test]
    fn pbt_single_reference_inline_output_remains_parseable(
        name in "[a-z][a-z0-9]{0,8}",
        param in "[a-z][a-z0-9]{0,8}",
        argument in "[-]?[0-9]{1,4}",
        addend in "[-]?[0-9]{1,4}",
    ) {
        let input = format!("(defun {name} ({param}) (+ {param} {addend}))\n(print ({name} {argument}))");
        let plan = plan_inline_function(InlineFunctionRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            definition_path: path("0"),
            call_paths: vec![path("1.1")],
            all_calls: false,
            remove_definition: false,
            allow_duplicate_evaluation: false,
            allow_drop_arguments: false,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(plan.calls[0].parameters[0].reference_count, 1);
        prop_assert_eq!(
            &plan.rewritten,
            &format!("(defun {name} ({param}) (+ {param} {addend}))\n(print (+ {argument} {addend}))")
        );
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }

    #[test]
    fn pbt_duplicate_evaluation_policy_controls_multi_reference_rewrites(
        name in "[a-z][a-z0-9]{0,8}",
        param in "[a-z][a-z0-9]{0,8}",
        callee in "[a-z][a-z0-9]{0,8}",
    ) {
        let input = format!("(defun {name} ({param}) (+ {param} {param}))\n(print ({name} ({callee})))");
        let default_result = plan_inline_function(InlineFunctionRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            definition_path: path("0"),
            call_paths: vec![path("1.1")],
            all_calls: false,
            remove_definition: false,
            allow_duplicate_evaluation: false,
            allow_drop_arguments: false,
        });
        prop_assert!(default_result.is_err());

        let plan = plan_inline_function(InlineFunctionRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            definition_path: path("0"),
            call_paths: vec![path("1.1")],
            all_calls: false,
            remove_definition: false,
            allow_duplicate_evaluation: true,
            allow_drop_arguments: false,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(plan.calls[0].parameters[0].reference_count, 2);
        prop_assert_eq!(
            &plan.rewritten,
            &format!("(defun {name} ({param}) (+ {param} {param}))\n(print (+ ({callee}) ({callee})))")
        );
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }
}
