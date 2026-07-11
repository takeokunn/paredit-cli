use super::*;

#[test]
fn rejects_swap_parameter_across_common_lisp_lambda_list_section() {
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
    .expect_err("section crossing must fail");

    assert!(
        error
            .to_string()
            .contains("cannot move 'b' across Common Lisp lambda-list sections")
    );
}
