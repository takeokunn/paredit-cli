use proptest::{prelude::*, test_runner::TestCaseError};

use super::{
    AddFunctionParameterRequest, FunctionParameterInsert, MoveFunctionParameterRequest,
    RemoveFunctionParameterRequest, plan_add_function_parameter, plan_move_function_parameter,
    plan_remove_function_parameter,
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
}
