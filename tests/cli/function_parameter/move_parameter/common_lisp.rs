use super::*;

#[test]
fn cli_plans_move_function_parameter_for_common_lisp() {
    let output = run_move_with_stdin(
        &[
            "--definition-path",
            "0",
            "--name",
            "scale",
            "--to-index",
            "2",
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
    let report = parse_move_function_parameter_report(&output.stdout).expect("parse move report");
    assert_eq!(report.function_name, "area");
    assert_eq!(report.parameter_name, "scale");
    assert_eq!(report.from_index, 0);
    assert_eq!(report.to_index, 2);
    assert_eq!(report.moved_arguments, vec!["2".to_owned()]);
    assert!(report.changed);
    assert!(!report.written);
    assert_eq!(
        report.rewritten,
        "(defun area (width height scale) (* scale width height))\n(defun render () (area 10 20 2))"
    );
}

#[test]
fn cli_plans_move_function_parameter_for_common_lisp_macrolet_binding() {
    assert_move_stdout(
        &common_lisp_move_args("0.3.1.0", "margin", "0", "0.3.2"),
        "(defun outer ()\n  (macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"from_index\": 2",
            "\"to_index\": 0",
            "(macrolet ((render (margin width height) `(list ,width ,height ,margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_common_lisp_cl_user_macrolet_binding() {
    assert_move_stdout(
        &common_lisp_move_args("0.3.1.0", "margin", "0", "0.3.2"),
        "(defun outer ()\n  (cl-user:macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"from_index\": 2",
            "\"to_index\": 0",
            "(cl-user:macrolet ((render (margin width height) `(list ,width ,height ,margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_common_lisp_labels_binding() {
    assert_move_stdout(
        &common_lisp_move_args("0.3.1.0", "margin", "0", "0.3.2"),
        "(defun outer ()\n  (labels ((render (width height margin) (list width height margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"from_index\": 2",
            "\"to_index\": 0",
            "(labels ((render (margin width height) (list width height margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_common_lisp_compiler_macrolet_binding() {
    assert_move_stdout(
        &common_lisp_move_args("0.3.1.0", "margin", "0", "0.3.2"),
        "(defun outer ()\n  (compiler-macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"from_index\": 2",
            "\"to_index\": 0",
            "(compiler-macrolet ((render (margin width height) `(list ,width ,height ,margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_common_lisp_cl_user_compiler_macrolet_binding() {
    assert_move_stdout(
        &common_lisp_move_args("0.3.1.0", "margin", "0", "0.3.2"),
        "(defun outer ()\n  (cl-user:compiler-macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"from_index\": 2",
            "\"to_index\": 0",
            "(cl-user:compiler-macrolet ((render (margin width height) `(list ,width ,height ,margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_common_lisp_define_setf_expander_call() {
    let output = run_move_with_stdin(
        &common_lisp_move_args("0", "extra", "1", "1"),
        "(define-setf-expander access (object mode extra)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw :fast) 1)",
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_move_function_parameter_report(&output.stdout).expect("parse move report");
    assert_eq!(report.function_name, "access");
    assert_eq!(report.parameter_name, "extra");
    assert_eq!(report.from_index, 2);
    assert_eq!(report.to_index, 1);
    assert_eq!(report.moved_arguments, vec![":fast".to_owned()]);
    assert_eq!(
        report.rewritten,
        "(define-setf-expander access (object extra mode)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :fast :rw) 1)"
    );
}

#[test]
fn cli_plans_move_function_parameter_for_common_lisp_long_form_defsetf_call() {
    assert_move_stdout(
        &common_lisp_move_args("0", "extra", "1", "1"),
        "(defsetf access (object mode extra) (value)\n  `(update-access ,object ,value))\n(setf (access item :rw :fast) 1)",
        &[
            "\"function_name\": \"access\"",
            "\"parameter_name\": \"extra\"",
            "\"from_index\": 2",
            "\"to_index\": 1",
            "\"moved_arguments\": [\n    \":fast\"\n  ]",
            "(defsetf access (object extra mode) (value)",
            "(setf (access item :fast :rw) 1)",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_define_setf_expander() {
    assert_move_stdout(
        &common_lisp_move_args("0", "extra", "1", "1"),
        "(define-setf-expander access (object mode extra)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw :fast) 1)",
        &[
            "\"function_name\": \"access\"",
            "\"parameter_name\": \"extra\"",
            "\"from_index\": 2",
            "\"to_index\": 1",
            "\"moved_arguments\": [\n    \":fast\"\n  ]",
            "(define-setf-expander access (object extra mode)",
            "(setf (access item :fast :rw) 1)",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_cl_qualified_define_setf_expander() {
    assert_move_stdout(
        &common_lisp_move_args("0", "extra", "1", "1"),
        "(cl:define-setf-expander access (object mode extra)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw :fast) 1)\n",
        &[
            "\"function_name\": \"access\"",
            "\"parameter_name\": \"extra\"",
            "\"to_index\": 1",
            "\"moved_arguments\": [\n    \":fast\"\n  ]",
            "(cl:define-setf-expander access (object extra mode)",
            "(setf (access item :fast :rw) 1)",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_define_modify_macro() {
    assert_move_stdout(
        &common_lisp_move_args("0", "extra", "1", "1"),
        "(define-modify-macro appendf (item tail extra) append)\n(appendf place 10 20 30)",
        &[
            "\"function_name\": \"appendf\"",
            "\"parameter_name\": \"extra\"",
            "\"from_index\": 2",
            "\"to_index\": 1",
            "\"moved_arguments\": [\n    \"30\"\n  ]",
            "(define-modify-macro appendf (item extra tail) append)",
            "(appendf place 10 30 20)",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_common_lisp_optional_parameter() {
    assert_move_stdout(
        &[
            "--definition-path",
            "0",
            "--name",
            "height",
            "--to-index",
            "1",
            "--call-path",
            "1.3",
            "--output",
            "json",
        ],
        "(defun area (scale &optional width height) (* scale width height))\n(defun render () (area 2 10 20))",
        &[
            "\"parameter_name\": \"height\"",
            "\"from_index\": 2",
            "\"to_index\": 1",
            "(defun area (scale &optional height width) (* scale width height))",
            "(defun render () (area 2 20 10))",
        ],
    );
}

#[test]
fn cli_plans_move_function_parameter_for_common_lisp_keyword_parameter() {
    assert_move_stdout(
        &common_lisp_move_args("0", "height", "1", "1.3"),
        "(defun render (node &key width height) (list node width height))\n(defun draw () (render widget :width 10 :height 20))",
        &[
            "\"parameter_name\": \"height\"",
            "\"from_index\": 2",
            "\"to_index\": 1",
            "(defun render (node &key height width) (list node width height))",
            "\"moved_arguments\": [\n    \"20\"\n  ]",
            "(defun draw () (render widget :height 20 :width 10))",
        ],
    );
}
