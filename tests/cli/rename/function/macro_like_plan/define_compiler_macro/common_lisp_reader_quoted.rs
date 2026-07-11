use super::super::*;

#[test]
fn cli_plans_common_lisp_define_compiler_macro_rename_inside_reader_quoted_lambda_body() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-compiler-macro-reader-quoted-lambda-body-plan",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"path\": \"0.3.2.1.0.2.1\"",
            "\"path\": \"0.3.2.1.0.2.2.1\"",
            "\"path\": \"0.3.2.1.0.2.3.0\"",
            "\"replacement\": \"optimized-add\"",
            "\"rewritten\": \"(define-compiler-macro optimized-add (x y) `(+ ,x ,y))\\n\"",
            "\"rewritten\": \"(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1))))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))\n",
            },
        ],
    });
}

#[test]
fn cli_plans_common_lisp_qualified_define_compiler_macro_rename_inside_reader_quoted_lambda_body() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-qualified-compiler-macro-reader-quoted-lambda-body-plan",
        from: "fast-add",
        to: "optimized-add",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (cl:compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"path\": \"0.3.2.1.0.2.1\"",
            "\"path\": \"0.3.2.1.0.2.2.1\"",
            "\"path\": \"0.3.2.1.0.2.3.0\"",
            "\"replacement\": \"optimized-add\"",
            "\"rewritten\": \"(cl:define-compiler-macro optimized-add (x y) `(+ ,x ,y))\\n\"",
            "\"rewritten\": \"(defun caller () #'(lambda () (cl:compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1))))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (cl:compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))\n",
            },
        ],
    });
}

#[test]
fn cli_plans_common_lisp_user_qualified_define_compiler_macro_rename_inside_reader_quoted_lambda_body(
) {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-user-qualified-compiler-macro-reader-quoted-lambda-body-plan",
        from: "fast-add",
        to: "optimized-add",
        input_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (cl-user:compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"path\": \"0.3.2.1.0.2.1\"",
            "\"path\": \"0.3.2.1.0.2.2.1\"",
            "\"path\": \"0.3.2.1.0.2.3.0\"",
            "\"replacement\": \"optimized-add\"",
            "\"rewritten\": \"(cl-user:define-compiler-macro optimized-add (x y) `(+ ,x ,y))\\n\"",
            "\"rewritten\": \"(defun caller () #'(lambda () (cl-user:compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1))))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () (cl-user:compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))\n",
            },
        ],
    });
}
