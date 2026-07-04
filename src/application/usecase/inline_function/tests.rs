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
fn inlines_common_lisp_optional_parameter_when_argument_is_supplied() {
    let input = "(defun add-default (x &optional (y 10)) (+ x y))\n(print (add-default 1 2))";
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

    assert_eq!(plan.calls[0].replacement, "(+ 1 2)");
    assert_eq!(
        plan.rewritten,
        "(defun add-default (x &optional (y 10)) (+ x y))\n(print (+ 1 2))"
    );
}

#[test]
fn rejects_common_lisp_optional_parameter_when_argument_is_missing() {
    let input = "(defun add-default (x &optional (y 10)) (+ x y))\n(print (add-default 1))";
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
    .expect_err("missing optional argument must fail");

    assert!(error.to_string().contains("arity mismatch"));
}

#[test]
fn rejects_common_lisp_optional_supplied_p_parameter() {
    let input = "(defun maybe (x &optional (y 10 y-p)) (if y-p y x))\n(print (maybe 1 2))";
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
    .expect_err("supplied-p optional parameter must fail");

    assert!(
        error
            .to_string()
            .contains("does not support &optional supplied-p")
    );
}

#[test]
fn inlines_common_lisp_key_parameter_when_argument_is_supplied() {
    let input =
        "(defun render (x &key (style :plain)) (list x style))\n(print (render 1 :style :bold))";
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

    assert_eq!(plan.calls[0].replacement, "(list 1 :bold)");
    assert_eq!(
        plan.rewritten,
        "(defun render (x &key (style :plain)) (list x style))\n(print (list 1 :bold))"
    );
    assert_eq!(plan.calls[0].parameters[1].name, "style");
    assert_eq!(plan.calls[0].parameters[1].argument, ":bold");
}

#[test]
fn inlines_common_lisp_external_key_parameter_designator() {
    let input = "(defun render (x &key ((:external internal) 10)) (list x internal))\n(print (render 1 :external 20))";
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

    assert_eq!(plan.calls[0].replacement, "(list 1 20)");
    assert_eq!(
        plan.rewritten,
        "(defun render (x &key ((:external internal) 10)) (list x internal))\n(print (list 1 20))"
    );
}

#[test]
fn rejects_common_lisp_key_parameter_when_argument_is_missing() {
    let input = "(defun render (x &key (style :plain)) (list x style))\n(print (render 1))";
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
    .expect_err("missing key argument must fail");

    assert!(
        error
            .to_string()
            .contains("must explicitly supply keyword :style")
    );
}

#[test]
fn rejects_common_lisp_key_parameter_with_duplicate_argument() {
    let input = "(defun render (x &key style) (list x style))\n(print (render 1 :style :bold :style :plain))";
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
    .expect_err("duplicate key argument must fail");

    assert!(error.to_string().contains("duplicate keyword :style"));
}

#[test]
fn rejects_common_lisp_key_supplied_p_parameter() {
    let input = "(defun render (x &key (style :plain style-p)) (if style-p style x))\n(print (render 1 :style :bold))";
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
    .expect_err("supplied-p key parameter must fail");

    assert!(
        error
            .to_string()
            .contains("does not support &key supplied-p")
    );
}

#[test]
fn discovers_all_calls_skips_common_lisp_flet_body_local_callable_calls() {
    let input = "(defun helper (x) (+ x 1))\n(defun render () (flet ((helper (x) (helper x))) (helper 2)) (helper 3))";
    let plan = plan_inline_function(InlineFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        call_paths: Vec::new(),
        all_calls: true,
        remove_definition: false,
        allow_duplicate_evaluation: false,
        allow_drop_arguments: false,
    })
    .expect("plan");

    assert_eq!(
        plan.call_paths
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["1.3.1.0.2", "1.4"]
    );
    assert_eq!(
        plan.rewritten,
        "(defun helper (x) (+ x 1))\n(defun render () (flet ((helper (x) (+ x 1))) (helper 2)) (+ 3 1))"
    );
}

#[test]
fn discovers_all_calls_skips_common_lisp_labels_local_callable_calls() {
    let input = "(defun helper (x) (+ x 1))\n(defun render () (labels ((helper (x) (if x (helper nil) 0))) (helper t)) (helper 3))";
    let plan = plan_inline_function(InlineFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        call_paths: Vec::new(),
        all_calls: true,
        remove_definition: false,
        allow_duplicate_evaluation: false,
        allow_drop_arguments: false,
    })
    .expect("plan");

    assert_eq!(
        plan.call_paths
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["1.4"]
    );
    assert_eq!(
        plan.rewritten,
        "(defun helper (x) (+ x 1))\n(defun render () (labels ((helper (x) (if x (helper nil) 0))) (helper t)) (+ 3 1))"
    );
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
