use super::*;

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp() {
    let output = run_swap_with_stdin(
        &[
            "--definition-path",
            "0",
            "--left-name",
            "scale",
            "--right-name",
            "height",
            "--call-path",
            "1.3",
            "--output",
            "json",
        ],
        "(defun area (scale width height) (* scale width height))\n(defun render () (area 2 10 20))",
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_swap_function_parameters_report(&output.stdout).expect("parse swap report");
    assert_eq!(report.function_name, "area");
    assert_eq!(report.left_name, "scale");
    assert_eq!(report.right_name, "height");
    assert_eq!(report.left_index, 0);
    assert_eq!(report.right_index, 2);
    assert_eq!(
        report.swapped_arguments,
        vec![CliSwappedArgumentReport {
            left: "2".to_owned(),
            right: "20".to_owned(),
        }]
    );
    assert!(report.changed);
    assert!(!report.written);
    assert_eq!(
        report.rewritten,
        "(defun area (height width scale) (* scale width height))\n(defun render () (area 20 10 2))"
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp_macrolet_binding() {
    assert_swap_stdout(
        &common_lisp_swap_args("0.3.1.0", "message", "context", "0.3.2"),
        "(defun outer ()\n  (macrolet ((with-log (message context) `(list ,message ,context)))\n    (with-log \"hello\" :ctx)))",
        &[
            "\"function_name\": \"with-log\"",
            "\"left_name\": \"message\"",
            "\"right_name\": \"context\"",
            "\"left_index\": 0",
            "\"right_index\": 1",
            "(macrolet ((with-log (context message) `(list ,message ,context)))",
            "(with-log :ctx \\\"hello\\\")",
        ],
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp_cl_user_macrolet_binding() {
    assert_swap_stdout(
        &common_lisp_swap_args("0.3.1.0", "message", "context", "0.3.2"),
        "(defun outer ()\n  (cl-user:macrolet ((with-log (message context) `(list ,message ,context)))\n    (with-log \"hello\" :ctx)))",
        &[
            "\"function_name\": \"with-log\"",
            "\"left_name\": \"message\"",
            "\"right_name\": \"context\"",
            "\"left_index\": 0",
            "\"right_index\": 1",
            "(cl-user:macrolet ((with-log (context message) `(list ,message ,context)))",
            "(with-log :ctx \\\"hello\\\")",
        ],
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp_compiler_macrolet_binding() {
    assert_swap_stdout(
        &common_lisp_swap_args("0.3.1.0", "message", "context", "0.3.2"),
        "(defun outer ()\n  (compiler-macrolet ((with-log (message context) `(list ,message ,context)))\n    (with-log \"hello\" :ctx)))",
        &[
            "\"function_name\": \"with-log\"",
            "\"left_name\": \"message\"",
            "\"right_name\": \"context\"",
            "\"left_index\": 0",
            "\"right_index\": 1",
            "(compiler-macrolet ((with-log (context message) `(list ,message ,context)))",
            "(with-log :ctx \\\"hello\\\")",
        ],
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp_cl_user_compiler_macrolet_binding() {
    assert_swap_stdout(
        &common_lisp_swap_args("0.3.1.0", "message", "context", "0.3.2"),
        "(defun outer ()\n  (cl-user:compiler-macrolet ((with-log (message context) `(list ,message ,context)))\n    (with-log \"hello\" :ctx)))",
        &[
            "\"function_name\": \"with-log\"",
            "\"left_name\": \"message\"",
            "\"right_name\": \"context\"",
            "\"left_index\": 0",
            "\"right_index\": 1",
            "(cl-user:compiler-macrolet ((with-log (context message) `(list ,message ,context)))",
            "(with-log :ctx \\\"hello\\\")",
        ],
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp_define_setf_expander_call() {
    let output = run_swap_with_stdin(
        &common_lisp_swap_args("0", "mode", "extra", "1"),
        "(define-setf-expander access (object mode extra)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw :fast) 1)",
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_swap_function_parameters_report(&output.stdout).expect("parse swap report");
    assert_eq!(report.function_name, "access");
    assert_eq!(report.left_name, "mode");
    assert_eq!(report.right_name, "extra");
    assert_eq!(report.left_index, 1);
    assert_eq!(report.right_index, 2);
    assert_eq!(
        report.swapped_arguments,
        vec![CliSwappedArgumentReport {
            left: ":rw".to_owned(),
            right: ":fast".to_owned(),
        }]
    );
    assert_eq!(
        report.rewritten,
        "(define-setf-expander access (object extra mode)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :fast :rw) 1)"
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp_long_form_defsetf_call() {
    assert_swap_stdout(
        &common_lisp_swap_args("0", "mode", "extra", "1"),
        "(defsetf access (object mode extra) (value)\n  `(update-access ,object ,value))\n(setf (access item :rw :fast) 1)",
        &[
            "\"function_name\": \"access\"",
            "\"left_name\": \"mode\"",
            "\"right_name\": \"extra\"",
            "\"left_index\": 1",
            "\"right_index\": 2",
            "\"left\": \":rw\"",
            "\"right\": \":fast\"",
            "(defsetf access (object extra mode) (value)",
            "(setf (access item :fast :rw) 1)",
        ],
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_define_setf_expander() {
    assert_swap_stdout(
        &common_lisp_swap_args("0", "mode", "extra", "1"),
        "(define-setf-expander access (object mode extra)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw :fast) 1)",
        &[
            "\"function_name\": \"access\"",
            "\"left_name\": \"mode\"",
            "\"right_name\": \"extra\"",
            "\"left_index\": 1",
            "\"right_index\": 2",
            "\"left\": \":rw\"",
            "\"right\": \":fast\"",
            "(define-setf-expander access (object extra mode)",
            "(setf (access item :fast :rw) 1)",
        ],
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_cl_qualified_define_setf_expander() {
    assert_swap_stdout(
        &common_lisp_swap_args("0", "mode", "extra", "1"),
        "(cl:define-setf-expander access (object mode extra)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw :fast) 1)\n",
        &[
            "\"function_name\": \"access\"",
            "\"left_name\": \"mode\"",
            "\"right_name\": \"extra\"",
            "\"swapped_arguments\": [\n    {\n      \"left\": \":rw\",\n      \"right\": \":fast\"\n    }\n  ]",
            "(cl:define-setf-expander access (object extra mode)",
            "(setf (access item :fast :rw) 1)",
        ],
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_define_modify_macro() {
    assert_swap_stdout(
        &common_lisp_swap_args("0", "tail", "extra", "1"),
        "(define-modify-macro appendf (item tail extra) append)\n(appendf place 10 20 30)",
        &[
            "\"function_name\": \"appendf\"",
            "\"left_name\": \"tail\"",
            "\"right_name\": \"extra\"",
            "\"left_index\": 1",
            "\"right_index\": 2",
            "\"left\": \"20\"",
            "\"right\": \"30\"",
            "(define-modify-macro appendf (item extra tail) append)",
            "(appendf place 10 30 20)",
        ],
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp_optional_parameters() {
    assert_swap_stdout(
        &[
            "--definition-path",
            "0",
            "--left-name",
            "width",
            "--right-name",
            "height",
            "--call-path",
            "1.3",
            "--output",
            "json",
        ],
        "(defun area (scale &optional width height) (* scale width height))\n(defun render () (area 2 10 20))",
        &[
            "\"left_name\": \"width\"",
            "\"right_name\": \"height\"",
            "\"left_index\": 1",
            "\"right_index\": 2",
            "\"left\": \"10\"",
            "\"right\": \"20\"",
            "(defun area (scale &optional height width) (* scale width height))",
            "(defun render () (area 2 20 10))",
        ],
    );
}

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp_keyword_parameters() {
    assert_swap_stdout(
        &common_lisp_swap_args("0", "width", "height", "1.3"),
        "(defun render (node &key width height) (list node width height))\n(defun draw () (render widget :width 10 :height 20))",
        &[
            "\"left_name\": \"width\"",
            "\"right_name\": \"height\"",
            "\"left_index\": 1",
            "\"right_index\": 2",
            "(defun render (node &key height width) (list node width height))",
            "\"swapped_arguments\": [\n    {\n      \"left\": \"10\",\n      \"right\": \"20\"\n    }\n  ]",
            "(defun draw () (render widget :height 20 :width 10))",
        ],
    );
}
