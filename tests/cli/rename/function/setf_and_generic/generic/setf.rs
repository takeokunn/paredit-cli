use super::super::super::*;

#[test]
fn cli_writes_common_lisp_setf_generic_function_and_method_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-setf-generic",
        dialect: None,
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(defgeneric (setf accessor) (value object))\n(defmethod (setf accessor) (value (object widget)) #'(setf accessor) (function (setf accessor)) (setf (accessor object) value))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(setf (accessor thing) 1)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(defgeneric (setf slot-accessor) (value object))\n(defmethod (setf slot-accessor) (value (object widget)) #'(setf slot-accessor) (function (setf slot-accessor)) (setf (slot-accessor object) value))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(setf (slot-accessor thing) 1)\n",
            },
        ],
        expected_definition_count: 2,
        expected_call_count: 4,
    });
}

#[test]
fn cli_plans_common_lisp_setf_generic_function_and_method_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-setf-generic-plan",
        from: "accessor",
        to: "slot-accessor",
        input_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(defgeneric (setf accessor) (value object))\n(defmethod (setf accessor) (value (object widget)) #'(setf accessor) (function (setf accessor)) (setf (accessor object) value))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(setf (accessor thing) 1)\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 2",
            "\"callCount\": 4",
            "\"path\": \"0.1.1\"",
            "\"path\": \"1.1.1\"",
            "\"path\": \"1.3.1\"",
            "\"path\": \"1.4.1.1\"",
            "\"path\": \"1.5.1.0\"",
            "\"path\": \"0.1.0\"",
            "\"replacement\": \"slot-accessor\"",
        ],
        unchanged_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(defgeneric (setf accessor) (value object))\n(defmethod (setf accessor) (value (object widget)) #'(setf accessor) (function (setf accessor)) (setf (accessor object) value))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(setf (accessor thing) 1)\n",
            },
        ],
    });
}
