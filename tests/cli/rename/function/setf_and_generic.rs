use super::*;

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
