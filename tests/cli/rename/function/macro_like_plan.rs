use super::*;

#[test]
fn cli_plans_common_lisp_define_method_combination_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-method-combination-plan",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 2",
            "\"path\": \"0.1\"",
            "\"path\": \"0.3.1\"",
            "\"path\": \"0.3.2.1\"",
            "\"replacement\": \"compose-render\"",
            "\"rewritten\": \"(define-method-combination compose-render (pane theme) ((primary *)) (list pane theme primary))\\n\"",
            "\"rewritten\": \"(defun caller () (list #'compose-render (function compose-render) render-combination))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(define-method-combination render-combination (pane theme) ((primary *)) (list pane theme primary))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (list #'render-combination (function render-combination) render-combination))\n",
            },
        ],
    });
}

#[test]
fn cli_plans_common_lisp_explicit_callable_designator_forms_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-explicit-callable-designators-plan",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 6",
            "\"path\": \"0.1\"",
            "\"path\": \"0.3.1\"",
            "\"path\": \"0.3.2.1\"",
            "\"path\": \"0.3.3.1\"",
            "\"path\": \"0.3.4.1\"",
            "\"path\": \"0.3.5.1\"",
            "\"path\": \"0.3.6.1\"",
            "\"replacement\": \"renamed\"",
            "\"rewritten\": \"(defmacro renamed (x) `(list ,x))\\n\"",
            "\"rewritten\": \"(defun caller () (list #'renamed (function renamed) (macro-function 'renamed) (compiler-macro-function 'renamed) (symbol-function 'renamed) (fdefinition 'renamed) helper))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(defmacro helper (x) `(list ,x))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (list #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper) helper))\n",
            },
        ],
    });
}
