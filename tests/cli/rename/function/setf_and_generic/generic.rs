use super::super::*;

#[test]
fn cli_writes_common_lisp_generic_function_and_method_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-generic",
        dialect: None,
        from: "render",
        to: "draw",
        input_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(defgeneric render (node stream))\n(defmethod render ((node widget) stream) (render node stream))\n(defmethod render :around ((node panel) stream) #'render (function render) (render node stream))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(render thing out)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(defgeneric draw (node stream))\n(defmethod draw ((node widget) stream) (draw node stream))\n(defmethod draw :around ((node panel) stream) #'draw (function draw) (draw node stream))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(draw thing out)\n",
            },
        ],
        expected_definition_count: 3,
        expected_call_count: 5,
    });
}

#[test]
fn cli_plans_common_lisp_generic_function_and_method_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-generic-plan",
        from: "render",
        to: "draw",
        input_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(defgeneric render (node stream))\n(defmethod render ((node widget) stream) (render node stream))\n(defmethod render :around ((node panel) stream) #'render (function render) (render node stream))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(render thing out)\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 3",
            "\"callCount\": 5",
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
                path: "generic.lisp",
                contents: "(defgeneric render (node stream))\n(defmethod render ((node widget) stream) (render node stream))\n(defmethod render :around ((node panel) stream) #'render (function render) (render node stream))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(render thing out)\n",
            },
        ],
    });
}

#[test]
fn cli_writes_common_lisp_user_qualified_generic_function_and_method_rename() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-common-lisp-user-qualified-generic",
        dialect: None,
        from: "render",
        to: "draw",
        input_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(cl-user:defgeneric render (node stream))\n(cl-user:defmethod render ((node widget) stream) (render node stream))\n(cl-user:defmethod render :around ((node panel) stream) #'render (function render) (render node stream))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(render thing out)\n",
            },
        ],
        expected_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(cl-user:defgeneric draw (node stream))\n(cl-user:defmethod draw ((node widget) stream) (draw node stream))\n(cl-user:defmethod draw :around ((node panel) stream) #'draw (function draw) (draw node stream))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(draw thing out)\n",
            },
        ],
        expected_definition_count: 3,
        expected_call_count: 5,
    });
}

#[test]
fn cli_plans_common_lisp_user_qualified_generic_function_and_method_rename() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-common-lisp-user-qualified-generic-plan",
        from: "render",
        to: "draw",
        input_files: &[
            FixtureFile {
                path: "generic.lisp",
                contents: "(cl-user:defgeneric render (node stream))\n(cl-user:defmethod render ((node widget) stream) (render node stream))\n(cl-user:defmethod render :around ((node panel) stream) #'render (function render) (render node stream))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(render thing out)\n",
            },
        ],
        stdout_needles: &[
            "\"definitionCount\": 3",
            "\"callCount\": 5",
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
                path: "generic.lisp",
                contents: "(cl-user:defgeneric render (node stream))\n(cl-user:defmethod render ((node widget) stream) (render node stream))\n(cl-user:defmethod render :around ((node panel) stream) #'render (function render) (render node stream))\n",
            },
            FixtureFile {
                path: "caller.lisp",
                contents: "(render thing out)\n",
            },
        ],
    });
}

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
