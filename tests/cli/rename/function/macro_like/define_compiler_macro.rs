use super::super::*;

#[test]
fn cli_writes_common_lisp_define_compiler_macro_rename_inside_reader_quoted_lambda_body() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-compiler-macro-reader-quoted-lambda-body",
        dialect: None,
        from: "fast-add",
        to: "optimized-add",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-compiler-macro optimized-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1))))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_define_compiler_macro_rename_inside_reader_quoted_lambda_body()
 {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-compiler-macro-reader-quoted-lambda-body",
        dialect: None,
        from: "fast-add",
        to: "optimized-add",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-compiler-macro optimized-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1))))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_common_lisp_qualified_define_compiler_macro_rename_inside_reader_quoted_lambda_body()
{
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-qualified-compiler-macro-reader-quoted-lambda-body",
        dialect: None,
        from: "fast-add",
        to: "optimized-add",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:define-compiler-macro optimized-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1))))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_common_lisp_define_compiler_macro_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-compiler-macro",
        dialect: None,
        from: "fast-add",
        to: "optimized-add",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'fast-add (function fast-add) fast-add)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-compiler-macro optimized-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'optimized-add (function optimized-add) fast-add)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 2,
    });
}

#[test]
fn cli_writes_common_lisp_qualified_define_compiler_macro_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-qualified-compiler-macro",
        dialect: None,
        from: "fast-add",
        to: "optimized-add",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'fast-add (function fast-add) fast-add)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:define-compiler-macro optimized-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'optimized-add (function optimized-add) fast-add)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 2,
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_define_compiler_macro_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-compiler-macro",
        dialect: None,
        from: "fast-add",
        to: "optimized-add",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'fast-add (function fast-add) fast-add)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-compiler-macro optimized-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'optimized-add (function optimized-add) fast-add)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 2,
    });
}
