use proptest::{prelude::*, test_runner::TestCaseError};

use super::{
    AddFunctionParameterRequest, FunctionParameterInsert, MoveFunctionParameterRequest,
    RemoveFunctionParameterRequest, ReorderFunctionParametersRequest,
    SwapFunctionParametersRequest, plan_add_function_parameter, plan_move_function_parameter,
    plan_remove_function_parameter, plan_reorder_function_parameters,
    plan_swap_function_parameters,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SymbolName, SyntaxTree};

fn path(value: &str) -> Path {
    value.parse().expect("path")
}

fn symbol(value: &str) -> SymbolName {
    SymbolName::new(value.to_owned()).expect("symbol")
}

#[test]
fn adds_parameter_to_definition_and_call() {
    let input = "(defun area (w) w)\n(print (area 3))";
    let plan = plan_add_function_parameter(AddFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("h"),
        argument: "4".to_owned(),
        call_paths: vec![path("1.1")],
        all_calls: false,
        insert: FunctionParameterInsert::End,
    })
    .expect("plan");

    assert_eq!(plan.function_name.as_str(), "area");
    assert_eq!(plan.rewritten, "(defun area (w h) w)\n(print (area 3 4))");
    assert!(plan.changed);
}

#[test]
fn moves_parameter_and_call_argument() {
    let input = "(defun f (a b c) (list a b c))\n(print (f 1 2 3))";
    let plan = plan_move_function_parameter(MoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("c"),
        to_index: 0,
        call_paths: vec![path("1.1")],
        all_calls: false,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defun f (c a b) (list a b c))\n(print (f 3 1 2))"
    );
    assert_eq!(plan.moved_arguments, vec!["3"]);
}

#[test]
fn removes_parameter_and_call_argument() {
    let input = "(defun f (a b) (+ a b))\n(print (f 1 2))";
    let plan = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("b"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        allow_missing_argument: false,
    })
    .expect("plan");

    assert_eq!(plan.rewritten, "(defun f (a) (+ a b))\n(print (f 1))");
    assert_eq!(plan.removed_arguments, vec![Some("2".to_owned())]);
}

#[test]
fn swaps_parameters_and_call_arguments() {
    let input = "(defun f (a b c) (list a b c))\n(print (f 1 2 3))";
    let plan = plan_swap_function_parameters(SwapFunctionParametersRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        left_name: symbol("a"),
        right_name: symbol("c"),
        call_paths: vec![path("1.1")],
        all_calls: false,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defun f (c b a) (list a b c))\n(print (f 3 2 1))"
    );
    assert_eq!(plan.left_index, 0);
    assert_eq!(plan.right_index, 2);
    assert_eq!(
        plan.swapped_arguments,
        vec![("1".to_owned(), "3".to_owned())]
    );
}

#[test]
fn reorders_parameters_and_call_arguments() {
    let input = "(defun f (a b c) (list a b c))\n(print (f 1 2 3))";
    let plan = plan_reorder_function_parameters(ReorderFunctionParametersRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        parameter_order: vec![symbol("c"), symbol("a"), symbol("b")],
        call_paths: vec![path("1.1")],
        all_calls: false,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defun f (c a b) (list a b c))\n(print (f 3 1 2))"
    );
    assert_eq!(
        plan.old_parameter_order
            .iter()
            .map(SymbolName::as_str)
            .collect::<Vec<_>>(),
        vec!["a", "b", "c"]
    );
    assert_eq!(
        plan.new_parameter_order
            .iter()
            .map(SymbolName::as_str)
            .collect::<Vec<_>>(),
        vec!["c", "a", "b"]
    );
    assert_eq!(
        plan.reordered_arguments,
        vec![vec!["3".to_owned(), "1".to_owned(), "2".to_owned()]]
    );
}

#[test]
fn rejects_reorder_with_missing_parameter() {
    let input = "(defun f (a b c) (list a b c))";
    let error = plan_reorder_function_parameters(ReorderFunctionParametersRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        parameter_order: vec![symbol("c"), symbol("a")],
        call_paths: Vec::new(),
        all_calls: false,
    })
    .expect_err("missing parameter must fail");

    assert!(error.to_string().contains("definition has 3"));
}

#[test]
fn discovers_all_same_file_calls() {
    let input = "(defun f (a) a)\n(print (f 1))\n(print (f 2))";
    let plan = plan_add_function_parameter(AddFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("b"),
        argument: "0".to_owned(),
        call_paths: Vec::new(),
        all_calls: true,
        insert: FunctionParameterInsert::End,
    })
    .expect("plan");

    assert_eq!(plan.call_paths, vec![path("1.1"), path("2.1")]);
    assert_eq!(
        plan.rewritten,
        "(defun f (a b) a)\n(print (f 1 0))\n(print (f 2 0))"
    );
}

proptest! {
    #[test]
    fn pbt_add_parameter_output_remains_parseable(
        name in "[a-z][a-z0-9]{0,8}",
        param in "[a-z][a-z0-9]{0,8}",
        added in "[a-z][a-z0-9]{0,8}",
        value in "[-]?[0-9]{1,4}",
        argument in "[-]?[0-9]{1,4}",
    ) {
        prop_assume!(name != param);
        prop_assume!(name != added);
        prop_assume!(param != added);
        let input = format!("(defun {name} ({param}) {param})\n(print ({name} {value}))");
        let plan = plan_add_function_parameter(AddFunctionParameterRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            definition_path: path("0"),
            name: symbol(&added),
            argument: argument.clone(),
            call_paths: vec![path("1.1")],
            all_calls: false,
            insert: FunctionParameterInsert::End,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(
            &plan.rewritten,
            &format!("(defun {name} ({param} {added}) {param})\n(print ({name} {value} {argument}))")
        );
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }

    #[test]
    fn pbt_move_parameter_output_remains_parseable(
        name in "[a-z][a-z0-9]{0,8}",
        a in "[a-z][a-z0-9]{0,8}",
        b in "[a-z][a-z0-9]{0,8}",
        first in "[-]?[0-9]{1,4}",
        second in "[-]?[0-9]{1,4}",
    ) {
        prop_assume!(name != a);
        prop_assume!(name != b);
        prop_assume!(a != b);
        let input = format!("(defun {name} ({a} {b}) (list {a} {b}))\n(print ({name} {first} {second}))");
        let plan = plan_move_function_parameter(MoveFunctionParameterRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            definition_path: path("0"),
            name: symbol(&b),
            to_index: 0,
            call_paths: vec![path("1.1")],
            all_calls: false,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(
            &plan.rewritten,
            &format!("(defun {name} ({b} {a}) (list {a} {b}))\n(print ({name} {second} {first}))")
        );
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }

    #[test]
    fn pbt_remove_parameter_output_remains_parseable(
        name in "[a-z][a-z0-9]{0,8}",
        a in "[a-z][a-z0-9]{0,8}",
        b in "[a-z][a-z0-9]{0,8}",
        first in "[-]?[0-9]{1,4}",
        second in "[-]?[0-9]{1,4}",
    ) {
        prop_assume!(name != a);
        prop_assume!(name != b);
        prop_assume!(a != b);
        let input = format!("(defun {name} ({a} {b}) {a})\n(print ({name} {first} {second}))");
        let plan = plan_remove_function_parameter(RemoveFunctionParameterRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            definition_path: path("0"),
            name: symbol(&b),
            call_paths: vec![path("1.1")],
            all_calls: false,
            allow_missing_argument: false,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(
            &plan.rewritten,
            &format!("(defun {name} ({a}) {a})\n(print ({name} {first}))")
        );
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }

    #[test]
    fn pbt_swap_parameters_output_remains_parseable(
        name in "[a-z][a-z0-9]{0,8}",
        a in "[a-z][a-z0-9]{0,8}",
        b in "[a-z][a-z0-9]{0,8}",
        c in "[a-z][a-z0-9]{0,8}",
        first in "[-]?[0-9]{1,4}",
        second in "[-]?[0-9]{1,4}",
        third in "[-]?[0-9]{1,4}",
    ) {
        prop_assume!(name != a);
        prop_assume!(name != b);
        prop_assume!(name != c);
        prop_assume!(a != b);
        prop_assume!(a != c);
        prop_assume!(b != c);
        let input = format!("(defun {name} ({a} {b} {c}) (list {a} {b} {c}))\n(print ({name} {first} {second} {third}))");
        let plan = plan_swap_function_parameters(SwapFunctionParametersRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            definition_path: path("0"),
            left_name: symbol(&a),
            right_name: symbol(&c),
            call_paths: vec![path("1.1")],
            all_calls: false,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(
            &plan.rewritten,
            &format!("(defun {name} ({c} {b} {a}) (list {a} {b} {c}))\n(print ({name} {third} {second} {first}))")
        );
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }

    #[test]
    fn pbt_reorder_parameters_output_remains_parseable(
        name in "[a-z][a-z0-9]{0,8}",
        a in "[a-z][a-z0-9]{0,8}",
        b in "[a-z][a-z0-9]{0,8}",
        c in "[a-z][a-z0-9]{0,8}",
        first in "[-]?[0-9]{1,4}",
        second in "[-]?[0-9]{1,4}",
        third in "[-]?[0-9]{1,4}",
    ) {
        prop_assume!(name != a);
        prop_assume!(name != b);
        prop_assume!(name != c);
        prop_assume!(a != b);
        prop_assume!(a != c);
        prop_assume!(b != c);
        let input = format!("(defun {name} ({a} {b} {c}) (list {a} {b} {c}))\n(print ({name} {first} {second} {third}))");
        let plan = plan_reorder_function_parameters(ReorderFunctionParametersRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            definition_path: path("0"),
            parameter_order: vec![symbol(&c), symbol(&a), symbol(&b)],
            call_paths: vec![path("1.1")],
            all_calls: false,
        })
        .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(
            &plan.rewritten,
            &format!("(defun {name} ({c} {a} {b}) (list {a} {b} {c}))\n(print ({name} {third} {first} {second}))")
        );
        SyntaxTree::parse(&plan.rewritten)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
    }
}
