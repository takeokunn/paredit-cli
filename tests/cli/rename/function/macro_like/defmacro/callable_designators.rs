use super::super::super::*;

#[test]
fn cli_writes_common_lisp_explicit_callable_designator_forms_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-explicit-callable-designators",
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
                contents: "(defun caller () (list #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper) helper))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(defmacro renamed (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (list #'renamed (function renamed) (macro-function 'renamed) (compiler-macro-function 'renamed) (symbol-function 'renamed) (fdefinition 'renamed) helper))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 6,
    });
}

#[test]
fn cli_writes_common_lisp_macrolet_expander_callable_designators_only() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-macrolet-expander-callable-designators",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(defmacro helper (x) x)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (macrolet ((helper () (list #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))) (helper) #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(defmacro renamed (x) x)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (macrolet ((helper () (list #'renamed (function renamed) (macro-function 'renamed) (compiler-macro-function 'renamed) (symbol-function 'renamed) (fdefinition 'renamed)))) (helper) #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 6,
    });
}

#[test]
fn cli_writes_common_lisp_compiler_macrolet_expander_callable_designators_only() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-compiler-macrolet-expander-callable-designators",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(defmacro helper (x) x)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (compiler-macrolet ((helper () (list #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))) (helper) #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(defmacro renamed (x) x)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (compiler-macrolet ((helper () (list #'renamed (function renamed) (macro-function 'renamed) (compiler-macro-function 'renamed) (symbol-function 'renamed) (fdefinition 'renamed)))) (helper) #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 6,
    });
}
