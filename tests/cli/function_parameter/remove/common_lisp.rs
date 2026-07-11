pub(super) use super::*;

#[test]
fn cli_plans_remove_function_parameter_for_common_lisp_flet_binding() {
    let mut args = common_lisp_remove_args("0.3.1.0", "margin", "0.3.2");
    args.extend(["--output", "json"]);
    assert_remove_stdout(
        &args,
        "(defun outer ()\n  (flet ((render (width height margin) (list width height margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"parameter_index\": 2",
            "(flet ((render (width height) (list width height margin)))",
            "(render 10 20))",
        ],
    );
}

#[test]
fn cli_plans_remove_function_parameter_for_common_lisp_macrolet_binding() {
    let mut args = common_lisp_remove_args("0.3.1.0", "margin", "0.3.2");
    args.extend(["--output", "json"]);
    assert_remove_stdout(
        &args,
        "(defun outer ()\n  (macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"parameter_index\": 2",
            "(macrolet ((render (width height) `(list ,width ,height ,margin)))",
            "(render 10 20))",
        ],
    );
}

#[test]
fn cli_plans_remove_function_parameter_for_common_lisp_cl_user_macrolet_binding() {
    let mut args = common_lisp_remove_args("0.3.1.0", "margin", "0.3.2");
    args.extend(["--output", "json"]);
    assert_remove_stdout(
        &args,
        "(defun outer ()\n  (cl-user:macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"parameter_index\": 2",
            "(cl-user:macrolet ((render (width height) `(list ,width ,height ,margin)))",
            "(render 10 20))",
        ],
    );
}

#[test]
fn cli_plans_remove_function_parameter_for_common_lisp_compiler_macrolet_binding() {
    let mut args = common_lisp_remove_args("0.3.1.0", "margin", "0.3.2");
    args.extend(["--output", "json"]);
    assert_remove_stdout(
        &args,
        "(defun outer ()\n  (compiler-macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"parameter_index\": 2",
            "(compiler-macrolet ((render (width height) `(list ,width ,height ,margin)))",
            "(render 10 20))",
        ],
    );
}

#[test]
fn cli_plans_remove_function_parameter_for_common_lisp_cl_user_compiler_macrolet_binding() {
    let mut args = common_lisp_remove_args("0.3.1.0", "margin", "0.3.2");
    args.extend(["--output", "json"]);
    assert_remove_stdout(
        &args,
        "(defun outer ()\n  (cl-user:compiler-macrolet ((render (width height margin) `(list ,width ,height ,margin)))\n    (render 10 20 5)))",
        &[
            "\"function_name\": \"render\"",
            "\"parameter_name\": \"margin\"",
            "\"parameter_index\": 2",
            "(cl-user:compiler-macrolet ((render (width height) `(list ,width ,height ,margin)))",
            "(render 10 20))",
        ],
    );
}

#[test]
fn cli_plans_remove_function_parameter_for_common_lisp_define_setf_expander_call() {
    let mut args = common_lisp_remove_args("0", "mode", "1");
    args.extend(["--output", "json"]);
    let report = assert_remove_success_output(
        &args,
        "(define-setf-expander access (object mode)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item :rw) 1)",
    );

    assert_eq!(report.function_name, "access");
    assert_eq!(report.parameter_name, "mode");
    assert_eq!(report.parameter_index, 1);
    assert_eq!(report.removed_arguments, vec![Some(":rw".to_owned())]);
    assert_eq!(
        report.rewritten,
        "(define-setf-expander access (object)\n  (values nil nil nil `(setf-helper ,object)))\n(setf (access item) 1)"
    );
}

#[test]
fn cli_plans_remove_function_parameter_for_common_lisp_long_form_defsetf_call() {
    let mut args = common_lisp_remove_args("0", "mode", "1");
    args.extend(["--output", "json"]);
    assert_remove_stdout(
        &args,
        "(defsetf access (object mode) (value)\n  `(update-access ,object ,value))\n(setf (access item :rw) 1)",
        &[
            "\"function_name\": \"access\"",
            "\"parameter_name\": \"mode\"",
            "\"parameter_index\": 1",
            "\"removed_arguments\": [\n    \":rw\"\n  ]",
            "(defsetf access (object) (value)",
            "(setf (access item) 1)",
        ],
    );
}

#[test]
fn cli_plans_remove_function_parameter_for_cl_qualified_define_setf_expander() {
    let input = "(cl:define-setf-expander access (object slot)
  (declare (ignore object slot))
  (values nil nil nil nil nil))
(setf (access item :mode) 1)
";

    let mut args = common_lisp_remove_args("0", "slot", "1");
    args.extend(["--output", "json"]);
    assert_remove_stdout(
        &args,
        input,
        &[
            "\"function_name\": \"access\"",
            "\"parameter_name\": \"slot\"",
            "\"parameter_index\": 1",
            "\"removed_arguments\": [\n    \":mode\"\n  ]",
            "(cl:define-setf-expander access (object)",
            "(setf (access item) 1)",
        ],
    );
}

#[test]
fn cli_plans_remove_function_parameter_for_define_modify_macro() {
    let mut args = common_lisp_remove_args("0", "tail", "1");
    args.extend(["--output", "json"]);
    assert_remove_stdout(
        &args,
        "(define-modify-macro appendf (item tail) append)\n(appendf place 10 20)",
        &[
            "\"function_name\": \"appendf\"",
            "\"parameter_name\": \"tail\"",
            "\"parameter_index\": 1",
            "\"removed_arguments\": [\n    \"20\"\n  ]",
            "(define-modify-macro appendf (item) append)",
            "(appendf place 10)",
        ],
    );
}

#[test]
fn cli_plans_remove_function_parameter_for_define_setf_expander() {
    let mut args = common_lisp_remove_args("0", "slot", "1");
    args.extend(["--output", "json"]);
    assert_remove_stdout(
        &args,
        "(define-setf-expander access (object slot)\n  (declare (ignore object slot))\n  (values nil nil nil nil nil))\n(setf (access item :mode) 1)",
        &[
            "\"function_name\": \"access\"",
            "\"parameter_name\": \"slot\"",
            "\"parameter_index\": 1",
            "\"removed_arguments\": [\n    \":mode\"\n  ]",
            "(define-setf-expander access (object)",
            "(setf (access item) 1)",
        ],
    );
}

#[test]
fn cli_plans_remove_common_lisp_optional_parameter_when_call_argument_is_missing() {
    let mut args = common_lisp_remove_args("0", "b", "1.1");
    args.extend(["--allow-missing-argument", "--output", "json"]);
    let report = assert_remove_success_output(
        &args,
        "(defun f (a &optional (b 2 b-p) c) (list a b c))\n(print (f 1))",
    );

    assert_eq!(report.function_name, "f");
    assert_eq!(report.parameter_name, "b");
    assert_eq!(report.parameter_index, 1);
    assert_eq!(report.parameter_keyword, None);
    assert_eq!(report.removed_arguments, vec![None]);
    assert_eq!(
        report.rewritten,
        "(defun f (a &optional c) (list a b c))\n(print (f 1))"
    );
}

#[test]
fn cli_plans_remove_common_lisp_key_parameter_when_call_keyword_is_missing() {
    let mut args = common_lisp_remove_args("0", "b", "1.1");
    args.extend(["--allow-missing-argument", "--output", "json"]);
    let report = assert_remove_success_output(
        &args,
        "(defun f (a &key b c) (list a b c))\n(print (f 1 :c 30))",
    );

    assert_eq!(report.function_name, "f");
    assert_eq!(report.parameter_name, "b");
    assert_eq!(report.parameter_index, 1);
    assert_eq!(report.parameter_keyword.as_deref(), Some(":b"));
    assert_eq!(report.removed_arguments, vec![None]);
    assert_eq!(
        report.rewritten,
        "(defun f (a &key c) (list a b c))\n(print (f 1 :c 30))"
    );
}

#[test]
fn cli_plans_remove_common_lisp_dotted_tail_parameter_without_touching_calls() {
    let mut args = common_lisp_remove_args("0", "tail", "1");
    args.extend(["--output", "json"]);
    let report = assert_remove_success_output(
        &args,
        "(defun collect (head . tail) (list head tail))\n(collect 1 2 3)",
    );

    assert_eq!(report.function_name, "collect");
    assert_eq!(report.parameter_name, "tail");
    assert_eq!(report.parameter_keyword, None);
    assert_eq!(report.removed_arguments, vec![None]);
    assert_eq!(
        report.rewritten,
        "(defun collect (head) (list head tail))\n(collect 1 2 3)"
    );
}
