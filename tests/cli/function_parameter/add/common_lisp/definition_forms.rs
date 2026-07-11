use super::*;

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_defmethod() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "style",
            "--argument",
            ":fancy",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defmethod render :around ((node widget) stream) (draw node stream))\n(render thing out)",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"style\"",
            "(defmethod render :around ((node widget) stream style)",
            "(render thing out :fancy)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_defgeneric() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "context",
            "--argument",
            ":ctx",
            "--call-path",
            "2",
            "--output",
            "json",
        ],
        "(defgeneric render (node stream))\n(defmethod render ((node widget) stream) (draw node stream))\n(render thing out)",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"context\"",
            "(defgeneric render (node stream context))",
            "(render thing out :ctx)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_long_form_defsetf_call() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "mode",
            "--argument",
            ":rw",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(defsetf access (object) (value)\n  `(update-access ,object ,value))\n(setf (access item) 1)",
        &[
            "\"function_name\": \"access\"",
            "\"parameter_name\": \"mode\"",
            "(defsetf access (object mode) (value)",
            "(setf (access item :rw) 1)",
        ],
    );
}

#[test]
fn cli_rejects_add_function_parameter_for_common_lisp_short_form_defsetf() {
    assert_add_function_parameter_failure(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "mode",
            "--argument",
            ":rw",
            "--call-path",
            "1",
        ],
        "(defsetf access update-access)\n(setf (access item) 1)",
        &["does not support short-form defsetf"],
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_define_compiler_macro() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "value",
            "--argument",
            "10",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(define-compiler-macro collect-body (head &body body) `(list ,head value ,@body))\n(collect-body 1 (+ 2 3) (+ 4 5))",
        &[
            "\"parameter_name\": \"value\"",
            "(define-compiler-macro collect-body (head value &body body)",
            "(collect-body 1 10 (+ 2 3) (+ 4 5))",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_define_setf_expander_call() {
    let output = run_add_function_parameter(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "mode",
            "--argument",
            ":rw",
            "--call-path",
            "1",
            "--output",
            "json",
        ],
        "(define-setf-expander access (object)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item) 1)",
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_add_function_parameter_report(&output.stdout).expect("parse add report");
    assert_eq!(report.function_name, "access");
    assert_eq!(report.parameter_name, "mode");
    assert_eq!(report.argument, ":rw");
    assert_eq!(
        report.rewritten,
        "(define-setf-expander access (object mode)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw) 1)"
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_macrolet_binding() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0.3.1.0",
            "--name",
            "context",
            "--argument",
            ":ctx",
            "--call-path",
            "0.3.2",
            "--output",
            "json",
        ],
        "(defun outer ()\n  (macrolet ((with-log (message) `(list ,message)))\n    (with-log \"hello\")))",
        &[
            "\"parameter_name\": \"context\"",
            "(macrolet ((with-log (message context) `(list ,message)))",
            "(with-log \\\"hello\\\" :ctx)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_cl_user_macrolet_binding() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0.3.1.0",
            "--name",
            "context",
            "--argument",
            ":ctx",
            "--call-path",
            "0.3.2",
            "--output",
            "json",
        ],
        "(defun outer ()\n  (cl-user:macrolet ((with-log (message) `(list ,message)))\n    (with-log \"hello\")))",
        &[
            "\"function_name\": \"with-log\"",
            "\"parameter_name\": \"context\"",
            "(cl-user:macrolet ((with-log (message context) `(list ,message)))",
            "(with-log \\\"hello\\\" :ctx)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_cl_user_compiler_macrolet_binding() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0.3.1.0",
            "--name",
            "context",
            "--argument",
            ":ctx",
            "--call-path",
            "0.3.2",
            "--output",
            "json",
        ],
        "(defun outer ()\n  (cl-user:compiler-macrolet ((expand-it (form env) `(list ,form ,env)))\n    (expand-it input env)))",
        &[
            "\"function_name\": \"expand-it\"",
            "\"parameter_name\": \"context\"",
            "(cl-user:compiler-macrolet ((expand-it (form env context) `(list ,form ,env)))",
            "(expand-it input env :ctx)",
        ],
    );
}

#[test]
fn cli_adds_function_parameter_for_define_modify_macro() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "tail",
            "--argument",
            "20",
            "--call-path",
            "1",
        ],
        "(define-modify-macro appendf (item) append)\n(appendf place 10)",
        &[
            "\"parameter_name\": \"tail\"",
            "(define-modify-macro appendf (item tail) append)",
            "(appendf place 10 20)",
        ],
    );
}
