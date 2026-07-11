use super::*;

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
