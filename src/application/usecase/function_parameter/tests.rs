use proptest::{prelude::*, test_runner::TestCaseError};

use super::{
    plan_add_function_parameter, plan_move_function_parameter, plan_remove_function_parameter,
    plan_reorder_function_parameters, plan_swap_function_parameters, AddFunctionParameterRequest,
    FunctionParameterInsert, MoveFunctionParameterRequest, RemoveFunctionParameterRequest,
    ReorderFunctionParametersRequest, SwapFunctionParametersRequest,
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
fn adds_common_lisp_key_parameter_to_definition_and_call() {
    let input = "(defun render (node &key color) (list node color margin))\n(render item :color :red)";
    let plan = plan_add_function_parameter(AddFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("margin"),
        argument: "8".to_owned(),
        call_paths: vec![path("1")],
        all_calls: false,
        insert: FunctionParameterInsert::End,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defun render (node &key color margin) (list node color margin))\n(render item :color :red :margin 8)"
    );
}

#[test]
fn adds_common_lisp_key_parameter_before_allow_other_keys() {
    let input = "(defun render (node &key color &allow-other-keys) (list node color margin))\n(render item :color :red)";
    let plan = plan_add_function_parameter(AddFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("margin"),
        argument: "8".to_owned(),
        call_paths: vec![path("1")],
        all_calls: false,
        insert: FunctionParameterInsert::End,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defun render (node &key color margin &allow-other-keys) (list node color margin))\n(render item :color :red :margin 8)"
    );
}

#[test]
fn rejects_add_common_lisp_key_parameter_with_duplicate_call_keyword() {
    let input = "(defun render (node &key color) (list node color margin))\n(render item :margin 4)";
    let error = plan_add_function_parameter(AddFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("margin"),
        argument: "8".to_owned(),
        call_paths: vec![path("1")],
        all_calls: false,
        insert: FunctionParameterInsert::End,
    })
    .expect_err("duplicate keyword must fail");

    assert!(error
        .to_string()
        .contains("already contains keyword argument :margin"));
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
fn adds_parameter_to_common_lisp_defmethod_and_call() {
    let input =
        "(defmethod render :around ((node widget) stream) (draw node stream))\n(render thing out)";
    let plan = plan_add_function_parameter(AddFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("style"),
        argument: ":fancy".to_owned(),
        call_paths: vec![path("1")],
        all_calls: false,
        insert: FunctionParameterInsert::End,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defmethod render :around ((node widget) stream style) (draw node stream))\n(render thing out :fancy)"
    );
}

#[test]
fn removes_specialized_parameter_from_common_lisp_defmethod_and_call() {
    let input = "(defmethod render :around ((node widget) stream style) (draw stream style))\n(render thing out :fancy)";
    let plan = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("node"),
        call_paths: vec![path("1")],
        all_calls: false,
        allow_missing_argument: false,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defmethod render :around (stream style) (draw stream style))\n(render out :fancy)"
    );
    assert_eq!(plan.parameter_index, 0);
    assert_eq!(plan.removed_arguments, vec![Some("thing".to_owned())]);
}

#[test]
fn removes_common_lisp_optional_parameter_spec_and_call_argument() {
    let input = "(defun f (a &optional (b 2 b-p) c) (list a b c))\n(print (f 1 3 4))";
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

    assert_eq!(
        plan.rewritten,
        "(defun f (a &optional c) (list a b c))\n(print (f 1 4))"
    );
    assert_eq!(plan.parameter_index, 1);
    assert_eq!(plan.removed_arguments, vec![Some("3".to_owned())]);
}

#[test]
fn removes_common_lisp_optional_parameter_when_call_argument_is_missing() {
    let input = "(defun f (a &optional (b 2 b-p) c) (list a b c))\n(print (f 1))";
    let plan = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("b"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        allow_missing_argument: true,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defun f (a &optional c) (list a b c))\n(print (f 1))"
    );
    assert_eq!(plan.parameter_index, 1);
    assert_eq!(plan.removed_arguments, vec![None]);
}

#[test]
fn removes_common_lisp_key_parameter_and_call_keyword_argument() {
    let input = "(defun f (a &key (b 2) ((:external c) 3 c-p) d) (list a b c d))\n(print (f 1 :b 20 :external 30 :d 40))";
    let plan = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("c"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        allow_missing_argument: false,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defun f (a &key (b 2) d) (list a b c d))\n(print (f 1 :b 20 :d 40))"
    );
    assert_eq!(plan.parameter_keyword.as_deref(), Some(":external"));
    assert_eq!(
        plan.removed_arguments,
        vec![Some(":external 30".to_owned())]
    );
}

#[test]
fn removes_common_lisp_key_parameter_when_call_keyword_is_missing() {
    let input = "(defun f (a &key b c) (list a b c))\n(print (f 1 :c 30))";
    let plan = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("b"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        allow_missing_argument: true,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defun f (a &key c) (list a b c))\n(print (f 1 :c 30))"
    );
    assert_eq!(plan.parameter_keyword.as_deref(), Some(":b"));
    assert_eq!(plan.removed_arguments, vec![None]);
}

#[test]
fn rejects_common_lisp_key_parameter_named_as_keyword() {
    let input = "(defun f (a &key :b) (list a))\n(print (f 1 :b 20))";
    let error = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol(":b"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        allow_missing_argument: false,
    })
    .expect_err("keyword-named parameter must fail");

    assert!(error
        .to_string()
        .contains("currently supports only simple parameters"));
}

#[test]
fn rejects_common_lisp_key_parameter_with_non_keyword_designator() {
    let input = "(defun f (a &key ((external b) 2)) (list a b))\n(print (f 1 :external 20))";
    let error = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("b"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        allow_missing_argument: false,
    })
    .expect_err("non-keyword external designator must fail");

    assert!(error
        .to_string()
        .contains("currently supports only simple parameters"));
}

#[test]
fn rejects_common_lisp_parameter_after_allow_other_keys_before_next_marker() {
    let input = "(defun f (a &key b &allow-other-keys c) (list a b c))\n(print (f 1 :b 20 :c 30))";
    let error = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("c"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        allow_missing_argument: false,
    })
    .expect_err("parameter after &allow-other-keys must fail");

    assert!(error.to_string().contains("after &allow-other-keys"));
}

#[test]
fn rejects_duplicate_common_lisp_keyword_argument_removal() {
    let input = "(defun f (a &key b) (list a b))\n(print (f 1 :b 20 :b 30))";
    let error = plan_remove_function_parameter(RemoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("b"),
        call_paths: vec![path("1.1")],
        all_calls: false,
        allow_missing_argument: false,
    })
    .expect_err("duplicate keyword must fail");

    assert!(error.to_string().contains("duplicate keyword argument :b"));
}

#[test]
fn rejects_add_parameter_to_common_lisp_lambda_list_marker() {
    let input = "(defun f (a &optional b) (list a b))\n(print (f 1 2))";
    let error = plan_add_function_parameter(AddFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("c"),
        argument: "3".to_owned(),
        call_paths: vec![path("1.1")],
        all_calls: false,
        insert: FunctionParameterInsert::End,
    })
    .expect_err("lambda-list marker must fail");

    assert!(error
        .to_string()
        .contains("currently supports only flat positional parameter lists or existing Common Lisp &key parameter lists"));
}

#[test]
fn rejects_move_parameter_across_common_lisp_lambda_list_marker() {
    let input = "(defun f (a &optional b) (list a b))\n(print (f 1 2))";
    let error = plan_move_function_parameter(MoveFunctionParameterRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        name: symbol("b"),
        to_index: 0,
        call_paths: vec![path("1.1")],
        all_calls: false,
    })
    .expect_err("lambda-list marker must fail");

    assert!(error
        .to_string()
        .contains("currently supports only flat positional parameter lists"));
}

#[test]
fn rejects_swap_parameter_across_common_lisp_lambda_list_marker() {
    let input = "(defun f (a &optional b) (list a b))\n(print (f 1 2))";
    let error = plan_swap_function_parameters(SwapFunctionParametersRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        left_name: symbol("a"),
        right_name: symbol("b"),
        call_paths: vec![path("1.1")],
        all_calls: false,
    })
    .expect_err("lambda-list marker must fail");

    assert!(error
        .to_string()
        .contains("currently supports only flat positional parameter lists"));
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
fn reorders_common_lisp_defmethod_specialized_parameters_and_call_arguments() {
    let input = "(defmethod render :around ((node widget) stream style) (draw node stream style))\n(render thing out :fancy)";
    let plan = plan_reorder_function_parameters(ReorderFunctionParametersRequest {
        input,
        dialect: Dialect::CommonLisp,
        definition_path: path("0"),
        parameter_order: vec![symbol("style"), symbol("node"), symbol("stream")],
        call_paths: vec![path("1")],
        all_calls: false,
    })
    .expect("plan");

    assert_eq!(
        plan.rewritten,
        "(defmethod render :around (style (node widget) stream) (draw node stream style))\n(render :fancy thing out)"
    );
    assert_eq!(
        plan.old_parameter_order
            .iter()
            .map(SymbolName::as_str)
            .collect::<Vec<_>>(),
        vec!["node", "stream", "style"]
    );
    assert_eq!(
        plan.reordered_arguments,
        vec![vec![
            ":fancy".to_owned(),
            "thing".to_owned(),
            "out".to_owned()
        ]]
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

#[test]
fn discovers_all_calls_respects_common_lisp_flet_callable_shadowing() {
    let input = "\
(defun f (a) a)
(defun caller ()
  (flet ((f (x) (f x)))
    (f 1))
  (f 2))";
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

    assert_eq!(plan.call_paths, vec![path("1.3.1.0.2"), path("1.4")]);
    assert_eq!(
        plan.rewritten,
        "\
(defun f (a b) a)
(defun caller ()
  (flet ((f (x) (f x 0)))
    (f 1))
  (f 2 0))"
    );
}

#[test]
fn discovers_all_calls_respects_common_lisp_labels_callable_shadowing() {
    let input = "\
(defun f (a) a)
(defun caller ()
  (labels ((f (x) (f x))
           (g (y) (f y)))
    (f 1)
    (cl:print (f 2)))
  (f 3))";
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

    assert_eq!(plan.call_paths, vec![path("1.4")]);
    assert_eq!(
        plan.rewritten,
        "\
(defun f (a b) a)
(defun caller ()
  (labels ((f (x) (f x))
           (g (y) (f y)))
    (f 1)
    (cl:print (f 2)))
  (f 3 0))"
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
