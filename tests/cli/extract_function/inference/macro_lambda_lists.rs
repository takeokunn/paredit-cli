use super::assert_extract_function_inference;

#[test]
fn cli_infers_extract_function_params_without_common_lisp_lambda_list_init_bindings() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "common-lisp",
            "--path",
            "0.3",
            "--name",
            "build",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (fallback) (lambda (&optional (value (fallback value) supplied)) (list value supplied fallback)))",
        &["fallback"],
        &["fallback"],
        "(build fallback)",
        Some(
            "(defun build (fallback) (lambda (&optional (value (fallback value) supplied)) (list value supplied fallback)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_define_setf_expander_macro_lambda_list_bindings() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--name",
            "wrap-expander",
            "--infer-params",
            "--output",
            "json",
        ],
        "(define-setf-expander slot (&whole whole &environment env target) (list whole env target outer))",
        &["outer"],
        &["outer"],
        "(wrap-expander outer)",
        Some(
            "(defun wrap-expander (outer) (define-setf-expander slot (&whole whole &environment env target) (list whole env target outer)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_define_compiler_macro_lambda_list_bindings() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--name",
            "wrap-compiler-macro",
            "--infer-params",
            "--output",
            "json",
        ],
        "(define-compiler-macro render (&whole whole &environment env target) (list whole env target outer))",
        &["outer"],
        &["outer"],
        "(wrap-compiler-macro outer)",
        Some(
            "(defun wrap-compiler-macro (outer) (define-compiler-macro render (&whole whole &environment env target) (list whole env target outer)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_package_qualified_compiler_macro_lambda_list_bindings()
 {
    assert_extract_function_inference(
        &[
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--name",
            "wrap-compiler-macro",
            "--infer-params",
            "--output",
            "json",
        ],
        "(define-compiler-macro render (&whole whole &environment env cl:target) (list whole env target outer))",
        &["outer"],
        &["outer"],
        "(wrap-compiler-macro outer)",
        Some(
            "(defun wrap-compiler-macro (outer) (define-compiler-macro render (&whole whole &environment env cl:target) (list whole env target outer)))",
        ),
    );
}
