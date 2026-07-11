use super::*;

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp() {
    let output = run_reorder_with_stdin(
        &[
            "--definition-path",
            "0",
            "--call-path",
            "1.3",
            "--output",
            "json",
        ],
        &["height", "scale", "width"],
        "(defun area (scale width height) (* scale width height))\n(defun render () (area 2 10 20))",
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report =
        parse_reorder_function_parameters_report(&output.stdout).expect("parse reorder report");
    assert_eq!(report.function_name, "area");
    assert_eq!(
        report.old_parameter_order,
        vec!["scale".to_owned(), "width".to_owned(), "height".to_owned()]
    );
    assert_eq!(
        report.new_parameter_order,
        vec!["height".to_owned(), "scale".to_owned(), "width".to_owned()]
    );
    assert_eq!(
        report.reordered_arguments,
        vec![vec!["20".to_owned(), "2".to_owned(), "10".to_owned()]]
    );
    assert!(report.changed);
    assert!(!report.written);
    assert_eq!(
        report.rewritten,
        "(defun area (height scale width) (* scale width height))\n(defun render () (area 20 2 10))"
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_labels_binding() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0.3.1.0", "0.3.2"),
        &["margin", "width", "height"],
        "(defun outer ()\n  (labels ((render (width height margin) (list width height margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"old_parameter_order\": [\n    \"width\",\n    \"height\",\n    \"margin\"\n  ]",
            "\"new_parameter_order\": [\n    \"margin\",\n    \"width\",\n    \"height\"\n  ]",
            "(labels ((render (margin width height) (list width height margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_cl_user_labels_binding() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0.3.1.0", "0.3.2"),
        &["margin", "width", "height"],
        "(defun outer ()\n  (cl-user:labels ((render (width height margin) (list width height margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"old_parameter_order\": [\n    \"width\",\n    \"height\",\n    \"margin\"\n  ]",
            "\"new_parameter_order\": [\n    \"margin\",\n    \"width\",\n    \"height\"\n  ]",
            "(cl-user:labels ((render (margin width height) (list width height margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_macrolet_binding() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0.3.1.0", "0.3.2"),
        &["margin", "width", "height"],
        "(defun outer ()\n  (macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"old_parameter_order\": [\n    \"width\",\n    \"height\",\n    \"margin\"\n  ]",
            "\"new_parameter_order\": [\n    \"margin\",\n    \"width\",\n    \"height\"\n  ]",
            "(macrolet ((render (margin width height) `(list ,width ,height ,margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_cl_user_macrolet_binding() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0.3.1.0", "0.3.2"),
        &["margin", "width", "height"],
        "(defun outer ()\n  (cl-user:macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"old_parameter_order\": [\n    \"width\",\n    \"height\",\n    \"margin\"\n  ]",
            "\"new_parameter_order\": [\n    \"margin\",\n    \"width\",\n    \"height\"\n  ]",
            "(cl-user:macrolet ((render (margin width height) `(list ,width ,height ,margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_compiler_macrolet_binding() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0.3.1.0", "0.3.2"),
        &["margin", "width", "height"],
        "(defun outer ()\n  (compiler-macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"old_parameter_order\": [\n    \"width\",\n    \"height\",\n    \"margin\"\n  ]",
            "\"new_parameter_order\": [\n    \"margin\",\n    \"width\",\n    \"height\"\n  ]",
            "(compiler-macrolet ((render (margin width height) `(list ,width ,height ,margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_cl_user_compiler_macrolet_binding() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0.3.1.0", "0.3.2"),
        &["margin", "width", "height"],
        "(defun outer ()\n  (cl-user:compiler-macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"old_parameter_order\": [\n    \"width\",\n    \"height\",\n    \"margin\"\n  ]",
            "\"new_parameter_order\": [\n    \"margin\",\n    \"width\",\n    \"height\"\n  ]",
            "(cl-user:compiler-macrolet ((render (margin width height) `(list ,width ,height ,margin)))",
            "(render 5 10 20)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_define_setf_expander_call() {
    let output = run_reorder_with_stdin(
        &common_lisp_reorder_args("0", "1"),
        &["object", "extra", "mode"],
        "(define-setf-expander access (object mode extra)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw :fast) 1)",
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report =
        parse_reorder_function_parameters_report(&output.stdout).expect("parse reorder report");
    assert_eq!(report.function_name, "access");
    assert_eq!(
        report.old_parameter_order,
        vec!["object".to_owned(), "mode".to_owned(), "extra".to_owned()]
    );
    assert_eq!(
        report.new_parameter_order,
        vec!["object".to_owned(), "extra".to_owned(), "mode".to_owned()]
    );
    assert_eq!(
        report.reordered_arguments,
        vec![vec![
            "item".to_owned(),
            ":fast".to_owned(),
            ":rw".to_owned(),
        ]]
    );
    assert_eq!(
        report.rewritten,
        "(define-setf-expander access (object extra mode)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :fast :rw) 1)"
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_cl_qualified_define_setf_expander_call() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0", "1"),
        &["object", "extra", "mode"],
        "(cl:define-setf-expander access (object mode extra)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw :fast) 1)",
        &[
            "\"function_name\": \"access\"",
            "\"old_parameter_order\": [\n    \"object\",\n    \"mode\",\n    \"extra\"\n  ]",
            "\"new_parameter_order\": [\n    \"object\",\n    \"extra\",\n    \"mode\"\n  ]",
            "(cl:define-setf-expander access (object extra mode)",
            "(setf (access item :fast :rw) 1)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_long_form_defsetf_call() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0", "1"),
        &["object", "extra", "mode"],
        "(defsetf access (object mode extra) (value)\n  `(update-access ,object ,value))\n(setf (access item :rw :fast) 1)",
        &[
            "\"function_name\": \"access\"",
            "\"old_parameter_order\": [\n    \"object\",\n    \"mode\",\n    \"extra\"\n  ]",
            "\"new_parameter_order\": [\n    \"object\",\n    \"extra\",\n    \"mode\"\n  ]",
            "(defsetf access (object extra mode) (value)",
            "(setf (access item :fast :rw) 1)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_define_modify_macro() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0", "1"),
        &["item", "extra", "tail"],
        "(define-modify-macro appendf (item tail extra) append)\n(appendf place 10 20 30)",
        &[
            "\"function_name\": \"appendf\"",
            "\"old_parameter_order\": [\n    \"item\",\n    \"tail\",\n    \"extra\"\n  ]",
            "\"new_parameter_order\": [\n    \"item\",\n    \"extra\",\n    \"tail\"\n  ]",
            "\"reordered_arguments\": [\n    [\n      \"10\",\n      \"30\",\n      \"20\"\n    ]\n  ]",
            "(define-modify-macro appendf (item extra tail) append)",
            "(appendf place 10 30 20)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_cl_qualified_define_modify_macro() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0", "1"),
        &["item", "extra", "tail"],
        "(cl:define-modify-macro appendf (item tail extra) append)\n(appendf place 10 20 30)",
        &[
            "\"function_name\": \"appendf\"",
            "\"old_parameter_order\": [\n    \"item\",\n    \"tail\",\n    \"extra\"\n  ]",
            "\"new_parameter_order\": [\n    \"item\",\n    \"extra\",\n    \"tail\"\n  ]",
            "\"reordered_arguments\": [\n    [\n      \"10\",\n      \"30\",\n      \"20\"\n    ]\n  ]",
            "(cl:define-modify-macro appendf (item extra tail) append)",
            "(appendf place 10 30 20)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_defmethod() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0", "1"),
        &["style", "node", "stream"],
        "(defmethod render :around ((node widget) stream style) (draw node stream style))\n(render thing out :fancy)",
        &[
            "\"function_name\": \"render\"",
            "\"old_parameter_order\": [\n    \"node\",\n    \"stream\",\n    \"style\"\n  ]",
            "(defmethod render :around (style (node widget) stream)",
            "(render :fancy thing out)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_defgeneric() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0", "2"),
        &["context", "node", "stream"],
        "(defgeneric render (node stream context))\n(defmethod render ((node widget) stream) (draw node stream))\n(render thing out :fancy)",
        &[
            "\"function_name\": \"render\"",
            "\"old_parameter_order\": [\n    \"node\",\n    \"stream\",\n    \"context\"\n  ]",
            "(defgeneric render (context node stream))",
            "(render :fancy thing out)",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_optional_parameters() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0", "1.3"),
        &["scale", "height", "width"],
        "(defun area (scale &optional width height) (* scale width height))\n(defun render () (area 2 10 20))",
        &[
            "(defun area (scale &optional height width) (* scale width height))",
            "(defun render () (area 2 20 10))",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_keyword_parameters() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0", "1.3"),
        &["node", "height", "width"],
        "(defun render (node &key width height) (list node width height))\n(defun draw () (render widget :width 10 :height 20))",
        &[
            "(defun render (node &key height width) (list node width height))",
            "\"reordered_arguments\": [\n    [\n      \"widget\",\n      \":height\",\n      \"20\",\n      \":width\",\n      \"10\"\n    ]\n  ]",
            "(defun draw () (render widget :height 20 :width 10))",
        ],
    );
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_macro_body_parameters() {
    assert_reorder_stdout(
        &common_lisp_reorder_args("0", "1"),
        &["right", "left"],
        "(defmacro wrap (left right &body body) `(list ,left ,right ,@body))\n(wrap 1 2 (+ 3 4) (+ 5 6))",
        &[
            "(defmacro wrap (right left &body body) `(list ,left ,right ,@body))",
            "\"reordered_arguments\": [\n    [\n      \"2\",\n      \"1\",\n      \"(+ 3 4)\",\n      \"(+ 5 6)\"\n    ]\n  ]",
            "(wrap 2 1 (+ 3 4) (+ 5 6))",
        ],
    );
}
