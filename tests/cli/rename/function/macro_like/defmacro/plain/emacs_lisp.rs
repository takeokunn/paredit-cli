use super::super::super::super::*;

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
