use super::*;

#[test]
fn cli_writes_function_rename_inside_macrolet_expander_only() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-macrolet-expander-write",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n(defun caller () (macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun new-name (x) x)\n(defun caller () (macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_function_rename_inside_compiler_macrolet_expander_only() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-compiler-macrolet-expander-write",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n(defun caller () (compiler-macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun new-name (x) x)\n(defun caller () (compiler-macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}
