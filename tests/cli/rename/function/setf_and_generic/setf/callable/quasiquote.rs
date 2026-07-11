use super::super::super::super::*;

#[test]
fn cli_writes_common_lisp_setf_callable_rename_inside_quasiquote() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-setf-quasiquote",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () `(list ,#'(setf old-name) ,(function (setf old-name)) ,(fdefinition '(setf old-name))))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(define-setf-expander new-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () `(list ,#'(setf new-name) ,(function (setf new-name)) ,(fdefinition '(setf new-name))))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_setf_callable_rename_inside_quasiquote() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-setf-quasiquote-plan",
        from: "old-name",
        to: "new-name",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () `(list ,#'(setf old-name) ,(function (setf old-name)) ,(fdefinition '(setf old-name))))\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(define-setf-expander new-name",
            "#'(setf new-name)",
            "(function (setf new-name)) ,(fdefinition '(setf new-name))))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () `(list ,#'(setf old-name) ,(function (setf old-name)) ,(fdefinition '(setf old-name))))\n",
            },
        ],
    });
}
