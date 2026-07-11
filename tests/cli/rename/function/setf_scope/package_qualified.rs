use super::super::*;

#[test]
fn cli_writes_common_lisp_qualified_define_setf_expander_rename_inside_reader_quoted_lambda_body()
{
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-qualified-setf-expander-reader-lambda",
        dialect: None,
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:define-setf-expander slot-accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'slot-accessor (function slot-accessor) (setf (slot-accessor thing) 1)))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_qualified_define_setf_expander_rename_inside_reader_quoted_lambda_body()
{
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-qualified-setf-expander-reader-lambda-plan",
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"rewritten\": \"(cl:define-setf-expander slot-accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\\n\"",
            "#'(lambda () #'slot-accessor (function slot-accessor) (setf (slot-accessor thing) 1)))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_define_setf_expander_rename_inside_reader_quoted_lambda_body()
{
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-setf-expander-reader-lambda",
        dialect: None,
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:define-setf-expander slot-accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'slot-accessor (function slot-accessor) (setf (slot-accessor thing) 1)))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_user_qualified_define_setf_expander_rename_inside_reader_quoted_lambda_body()
{
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-user-qualified-setf-expander-reader-lambda-plan",
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"rewritten\": \"(cl-user:define-setf-expander slot-accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\\n\"",
            "#'(lambda () #'slot-accessor (function slot-accessor) (setf (slot-accessor thing) 1)))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
    });
}

#[test]
fn cli_writes_common_lisp_qualified_defsetf_rename_inside_reader_quoted_lambda_body() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-qualified-defsetf-reader-lambda",
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
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:defsetf slot-accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'slot-accessor (function slot-accessor) (setf (slot-accessor thing) 1)))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_qualified_defsetf_rename_inside_reader_quoted_lambda_body() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-qualified-defsetf-reader-lambda-plan",
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"rewritten\": \"(cl:defsetf slot-accessor set-accessor)\\n\"",
            "#'(lambda () #'slot-accessor (function slot-accessor) (setf (slot-accessor thing) 1)))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_defsetf_rename_inside_reader_quoted_lambda_body() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-defsetf-reader-lambda",
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
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:defsetf slot-accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'slot-accessor (function slot-accessor) (setf (slot-accessor thing) 1)))\n",
            },
        ],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_plans_common_lisp_user_qualified_defsetf_rename_inside_reader_quoted_lambda_body() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-user-qualified-defsetf-reader-lambda-plan",
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"rewritten\": \"(cl-user:defsetf slot-accessor set-accessor)\\n\"",
            "#'(lambda () #'slot-accessor (function slot-accessor) (setf (slot-accessor thing) 1)))\\n\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "accessor.lisp",
                contents: "(cl-user:defsetf accessor set-accessor)\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(defun caller () #'(lambda () #'accessor (function accessor) (setf (accessor thing) 1)))\n",
            },
        ],
    });
}
