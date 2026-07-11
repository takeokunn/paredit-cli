use super::super::*;

#[test]
fn cli_plans_common_lisp_define_compiler_macro_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-compiler-macro-plan",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 2",
            "\"path\": \"0.1\"",
            "\"path\": \"0.3\"",
            "\"path\": \"0.4.1\"",
            "\"replacement\": \"optimized-add\"",
            "\"rewritten\": \"(define-compiler-macro optimized-add (x y) `(+ ,x ,y))\\n\"",
            "\"rewritten\": \"(defun caller () #'optimized-add (function optimized-add) fast-add)\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'fast-add (function fast-add) fast-add)\n",
            },
        ],
    });
}

#[test]
fn cli_plans_common_lisp_qualified_define_compiler_macro_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-qualified-compiler-macro-plan",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 2",
            "\"path\": \"0.1\"",
            "\"path\": \"0.3\"",
            "\"path\": \"0.4.1\"",
            "\"replacement\": \"optimized-add\"",
            "\"rewritten\": \"(cl:define-compiler-macro optimized-add (x y) `(+ ,x ,y))\\n\"",
            "\"rewritten\": \"(defun caller () #'optimized-add (function optimized-add) fast-add)\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'fast-add (function fast-add) fast-add)\n",
            },
        ],
    });
}

#[test]
fn cli_plans_common_lisp_user_qualified_define_compiler_macro_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-user-qualified-compiler-macro-plan",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 2",
            "\"path\": \"0.1\"",
            "\"path\": \"0.3\"",
            "\"path\": \"0.4.1\"",
            "\"replacement\": \"optimized-add\"",
            "\"rewritten\": \"(cl-user:define-compiler-macro optimized-add (x y) `(+ ,x ,y))\\n\"",
            "\"rewritten\": \"(defun caller () #'optimized-add (function optimized-add) fast-add)\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'fast-add (function fast-add) fast-add)\n",
            },
        ],
    });
}
