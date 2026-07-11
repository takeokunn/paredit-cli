use super::super::*;

#[test]
fn cli_writes_common_lisp_setf_callable_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-setf",
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
                contents: "(defun caller () (setf (old-name place) 1) #'old-name (function old-name) old-name)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(define-setf-expander new-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (new-name place) 1) #'new-name (function new-name) old-name)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_setf_callable_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-setf-plan",
        from: "old-name",
        to: "new-name",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (old-name place) 1) #'old-name (function old-name) old-name)\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(define-setf-expander new-name",
            "\"path\": \"0.3.1.0\"",
            "(defun caller () (setf (new-name place) 1) #'new-name (function new-name) old-name)\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (old-name place) 1) #'old-name (function old-name) old-name)\n",
            },
        ],
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_setf_callable_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-setf",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (old-name place) 1) #'old-name (function old-name) old-name)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:define-setf-expander new-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (new-name place) 1) #'new-name (function new-name) old-name)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_user_qualified_setf_callable_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-user-setf-plan",
        from: "old-name",
        to: "new-name",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (old-name place) 1) #'old-name (function old-name) old-name)\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(cl-user:define-setf-expander new-name",
            "\"path\": \"0.3.1.0\"",
            "(defun caller () (setf (new-name place) 1) #'new-name (function new-name) old-name)\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (old-name place) 1) #'old-name (function old-name) old-name)\n",
            },
        ],
    });
}

#[test]
fn cli_writes_common_lisp_qualified_setf_callable_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-qualified-setf",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (old-name place) 1) #'old-name (function old-name) old-name)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:define-setf-expander new-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (new-name place) 1) #'new-name (function new-name) old-name)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_qualified_setf_callable_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-qualified-setf-plan",
        from: "old-name",
        to: "new-name",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (old-name place) 1) #'old-name (function old-name) old-name)\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(cl:define-setf-expander new-name",
            "\"path\": \"0.3.1.0\"",
            "(defun caller () (setf (new-name place) 1) #'new-name (function new-name) old-name)\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:define-setf-expander old-name (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (old-name place) 1) #'old-name (function old-name) old-name)\n",
            },
        ],
    });
}

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

#[test]
fn cli_writes_common_lisp_qualified_defsetf_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-qualified-defsetf",
        dialect: None,
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (accessor place) 1) #'accessor (function accessor) accessor)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:defsetf slot-accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (slot-accessor place) 1) #'slot-accessor (function slot-accessor) accessor)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_qualified_defsetf_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-qualified-defsetf-plan",
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (accessor place) 1) #'accessor (function accessor) accessor)\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(cl:defsetf slot-accessor set-accessor)\\n\"",
            "\"path\": \"0.3.1.0\"",
            "(defun caller () (setf (slot-accessor place) 1) #'slot-accessor (function slot-accessor) accessor)\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (accessor place) 1) #'accessor (function accessor) accessor)\n",
            },
        ],
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_defsetf_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-defsetf",
        dialect: None,
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (accessor place) 1) #'accessor (function accessor) accessor)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:defsetf slot-accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (slot-accessor place) 1) #'slot-accessor (function slot-accessor) accessor)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_user_qualified_defsetf_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-user-qualified-defsetf-plan",
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (accessor place) 1) #'accessor (function accessor) accessor)\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(cl-user:defsetf slot-accessor set-accessor)\\n\"",
            "\"path\": \"0.3.1.0\"",
            "(defun caller () (setf (slot-accessor place) 1) #'slot-accessor (function slot-accessor) accessor)\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (accessor place) 1) #'accessor (function accessor) accessor)\n",
            },
        ],
    });
}

#[test]
fn cli_writes_common_lisp_defsetf_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-defsetf",
        dialect: None,
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (accessor place) 1) #'accessor (function accessor) accessor)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(defsetf slot-accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (slot-accessor place) 1) #'slot-accessor (function slot-accessor) accessor)\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_defsetf_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-defsetf-plan",
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (accessor place) 1) #'accessor (function accessor) accessor)\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(defsetf slot-accessor set-accessor)\\n\"",
            "\"path\": \"0.3.1.0\"",
            "(defun caller () (setf (slot-accessor place) 1) #'slot-accessor (function slot-accessor) accessor)\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () (setf (accessor place) 1) #'accessor (function accessor) accessor)\n",
            },
        ],
    });
}
