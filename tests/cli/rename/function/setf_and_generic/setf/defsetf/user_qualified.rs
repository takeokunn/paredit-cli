use super::super::super::super::*;

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
