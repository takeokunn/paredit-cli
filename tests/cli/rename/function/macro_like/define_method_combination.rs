use super::super::*;

#[test]
fn cli_writes_common_lisp_define_method_combination_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-method-combination",
        dialect: None,
        from: "render-combination",
        to: "compose-render",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-method-combination render-combination (pane theme) ((primary *)) (list pane theme primary))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (list #'render-combination (function render-combination) render-combination))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-method-combination compose-render (pane theme) ((primary *)) (list pane theme primary))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (list #'compose-render (function compose-render) render-combination))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 2,
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_define_method_combination_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-method-combination",
        dialect: None,
        from: "render-combination",
        to: "compose-render",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-method-combination render-combination (pane theme) ((primary *)) (list pane theme primary))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (list #'render-combination (function render-combination) render-combination))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-method-combination compose-render (pane theme) ((primary *)) (list pane theme primary))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (list #'compose-render (function compose-render) render-combination))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 2,
    });
}
