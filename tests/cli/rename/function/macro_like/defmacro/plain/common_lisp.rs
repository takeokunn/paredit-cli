use super::super::super::super::*;

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
