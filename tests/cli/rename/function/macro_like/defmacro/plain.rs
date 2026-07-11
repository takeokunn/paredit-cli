use super::super::super::*;

#[test]
fn cli_writes_emacs_lisp_cl_defmacro_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-emacs-lisp-defmacro",
        dialect: Some("emacs-lisp"),
        from: "helper",
        to: "renamed",
        input_files: &[
            FixtureFile {
                path: "macro.el",
                contents: "(cl-defmacro helper (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.el",
                contents: "(defun caller () (helper 1) (helper 2) helper)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.el",
                contents: "(cl-defmacro renamed (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.el",
                contents: "(defun caller () (renamed 1) (renamed 2) helper)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 2,
    });
}

#[test]
fn cli_writes_common_lisp_defmacro_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-defmacro",
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
                contents: "(helper 1)\n(list #'helper helper)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(defmacro renamed (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(renamed 1)\n(list #'renamed helper)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 2,
    });
}

#[test]
fn cli_writes_common_lisp_qualified_defmacro_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-qualified-defmacro",
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
                contents: "(helper 1)\n(list #'helper helper)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:defmacro renamed (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(renamed 1)\n(list #'renamed helper)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 2,
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_defmacro_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-defmacro",
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
                contents: "(helper 1)\n(list #'helper helper)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:defmacro renamed (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(renamed 1)\n(list #'renamed helper)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 2,
    });
}

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
