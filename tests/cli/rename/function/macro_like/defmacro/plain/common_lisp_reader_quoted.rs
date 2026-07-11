use super::super::super::super::*;

#[test]
fn cli_writes_common_lisp_defmacro_rename_inside_reader_quoted_lambda_body() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-defmacro-reader-quoted-lambda-body",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(defmacro helper (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (macrolet ((helper (value) (list #'helper (function helper) (helper value)))) (helper 1))))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(defmacro renamed (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (macrolet ((helper (value) (list #'renamed (function renamed) (renamed value)))) (helper 1))))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_common_lisp_qualified_defmacro_rename_inside_reader_quoted_lambda_body() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-qualified-defmacro-reader-quoted-lambda-body",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:defmacro helper (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (macrolet ((helper (value) (list #'helper (function helper) (helper value)))) (helper 1))))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:defmacro renamed (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (macrolet ((helper (value) (list #'renamed (function renamed) (renamed value)))) (helper 1))))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_defmacro_rename_inside_reader_quoted_lambda_body() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-defmacro-reader-quoted-lambda-body",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:defmacro helper (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (macrolet ((helper (value) (list #'helper (function helper) (helper value)))) (helper 1))))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:defmacro renamed (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (macrolet ((helper (value) (list #'renamed (function renamed) (renamed value)))) (helper 1))))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}
