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
fn cli_plans_cl_user_define_method_combination_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-cl-user-method-combination-plan",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 2",
            "\"path\": \"0.1\"",
            "\"path\": \"0.3.1\"",
            "\"path\": \"0.3.2.1\"",
            "\"replacement\": \"compose-render\"",
            "\"rewritten\": \"(cl-user:define-method-combination compose-render (pane theme) ((primary *)) (list pane theme primary))\\n\"",
            "\"rewritten\": \"(defun caller () (list #'compose-render (function compose-render) render-combination))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "macro.lisp",
                contents: "(cl-user:define-method-combination render-combination (pane theme) ((primary *)) (list pane theme primary))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (list #'render-combination (function render-combination) render-combination))\n",
            },
        ],
    });
}
