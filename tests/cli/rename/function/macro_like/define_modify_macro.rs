use super::super::*;

#[test]
fn cli_writes_common_lisp_macro_like_callable_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-macro",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-modify-macro old-name (delta) +)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (old-name place 1) #'old-name (function old-name) old-name)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-modify-macro new-name (delta) +)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (new-name place 1) #'new-name (function new-name) old-name)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_common_lisp_qualified_define_modify_macro_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-qualified-modify-macro",
        dialect: None,
        from: "bumpf",
        to: "stepf",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:define-modify-macro bumpf (delta) +)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (bumpf place 1) #'bumpf (function bumpf) bumpf)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:define-modify-macro stepf (delta) +)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (stepf place 1) #'stepf (function stepf) bumpf)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_define_modify_macro_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-modify-macro",
        dialect: None,
        from: "bumpf",
        to: "stepf",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-modify-macro bumpf (delta) +)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (bumpf place 1) #'bumpf (function bumpf) bumpf)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-modify-macro stepf (delta) +)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (stepf place 1) #'stepf (function stepf) bumpf)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}
