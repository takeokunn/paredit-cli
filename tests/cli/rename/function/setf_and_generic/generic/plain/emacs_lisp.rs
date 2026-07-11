use super::super::super::super::*;

#[test]
fn cli_writes_emacs_lisp_generic_function_and_method_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-emacs-lisp-generic",
        dialect: Some("emacs-lisp"),
        from: "render",
        to: "draw",
        input_files: &[
            FixtureFile {
                path: "generic.el",
                contents: "(cl-defgeneric render (node stream))\n(cl-defmethod render ((node widget) stream) (render node stream))\n(cl-defmethod render :around ((node panel) stream) #'render (function render) (render node stream))\n",
            },
            FixtureFile {
                path: "caller.el",
                contents: "(render thing out)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "generic.el",
                contents: "(cl-defgeneric draw (node stream))\n(cl-defmethod draw ((node widget) stream) (draw node stream))\n(cl-defmethod draw :around ((node panel) stream) #'draw (function draw) (draw node stream))\n",
            },
            FixtureFile {
                path: "caller.el",
                contents: "(draw thing out)\n",
            },
        ],
        expected_definition_count: 3,
        expected_call_count: 5,
    });
}

#[test]
fn cli_plans_emacs_lisp_generic_function_and_method_rename() {
    assert_plan_case_with_dialect(
        PlanCase {
            fixture_name: "rename-function-emacs-lisp-generic-plan",
            from: "render",
            to: "draw",
            input_files: &[
                FixtureFile {
                    path: "generic.el",
                    contents: "(cl-defgeneric render (node stream))\n(cl-defmethod render ((node widget) stream) (render node stream))\n(cl-defmethod render :around ((node panel) stream) #'render (function render) (render node stream))\n",
                },
                FixtureFile {
                    path: "caller.el",
                    contents: "(render thing out)\n",
                },
            ],
            stdout_needles: &[
                "\"definitionCount\": 3",
                "\"callCount\": 5",
                "\"dialect\": \"emacs-lisp\"",
                "\"path\": \"0.1\"",
                "\"path\": \"1.1\"",
                "\"path\": \"2.1\"",
                "\"path\": \"1.3.0\"",
                "\"path\": \"2.4\"",
                "\"path\": \"2.5.1\"",
                "\"path\": \"2.6.0\"",
                "\"path\": \"0.0\"",
                "\"replacement\": \"draw\"",
            ],
            unchanged_files: &[
                FixtureFile {
                    path: "generic.el",
                    contents: "(cl-defgeneric render (node stream))\n(cl-defmethod render ((node widget) stream) (render node stream))\n(cl-defmethod render :around ((node panel) stream) #'render (function render) (render node stream))\n",
                },
                FixtureFile {
                    path: "caller.el",
                    contents: "(render thing out)\n",
                },
            ],
        },
        Some("emacs-lisp"),
    );
}
