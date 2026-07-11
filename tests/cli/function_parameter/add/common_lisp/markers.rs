use super::*;

#[test]
fn cli_plans_add_function_parameter_before_common_lisp_rest_marker() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "value",
            "--argument",
            "10",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun collect (head &rest tail) (list head value tail))\n(collect 1 2 3)",
        &[
            "\"parameter_name\": \"value\"",
            "(defun collect (head value &rest tail)",
            "(collect 1 10 2 3)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_before_common_lisp_dotted_tail() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "value",
            "--argument",
            "10",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defun collect (head . tail) (list head tail))\n(collect 1 2 3)",
        &[
            "\"parameter_section\": \"required\"",
            "(defun collect (head value . tail)",
            "(collect 1 10 2 3)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_before_common_lisp_body_marker_in_macro() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "value",
            "--argument",
            "10",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defmacro collect-body (head &body body) `(list ,head value ,@body))\n(collect-body 1 (+ 2 3) (+ 4 5))",
        &[
            "\"parameter_name\": \"value\"",
            "(defmacro collect-body (head value &body body)",
            "(collect-body 1 10 (+ 2 3) (+ 4 5))",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_with_hyphen_prefixed_argument() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--argument",
            "-10",
            "--call-path",
            "1.1",
            "--output",
            "json",
        ],
        "(defun area (width height) (* width height))\n(print (area 3 4))",
        &[
            "\"argument\": \"-10\"",
            "\"parameter_section\": \"required\"",
            "(defun area (width height margin)",
            "(print (area 3 4 -10))",
        ],
    );
}
