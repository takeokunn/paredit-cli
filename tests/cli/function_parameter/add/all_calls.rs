use super::*;

#[test]
fn cli_plans_add_function_parameter_with_all_calls() {
    let output = run_add_function_parameter(
        &[
            "add-function-parameter",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--argument",
            "5",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))\n(defun total () (+ (area 3 4) 1))",
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_add_function_parameter_report(&output.stdout).expect("parse add report");
    assert!(report.all_calls);
    assert_eq!(report.function_name, "area");
    assert_eq!(report.parameter_name, "margin");
    assert_eq!(
        report.rewritten,
        "(defun area (width height margin) (* width height))\n(defun render () (area 10 20 5))\n(defun total () (+ (area 3 4 5) 1))"
    );
}

#[test]
fn cli_plans_add_function_parameter_all_calls_without_common_lisp_labels_shadowed_calls() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "b",
            "--argument",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "\
(defun f (a) a)
(defun caller ()
  (labels ((f (x) (f x))
           (g (y) (f y)))
    (f 1)
    (cl:print (f 2)))
  (f 3))",
        &[
            "\"all_calls\": true",
            "\"call_paths\": [\n    \"1.4\"\n  ]",
            "(defun f (a b) a)",
            "(f x))",
            "(f 1)",
            "(f 3 0)",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_all_calls_respects_common_lisp_macrolet_shadowing() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "b",
            "--argument",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "\
(defun f (a) a)
(defun caller ()
  (macrolet ((f (x) (f x)))
    (f 1))
  (f 2))",
        &[
            "\"all_calls\": true",
            "\"call_paths\": [\n    \"1.3.1.0.2\",\n    \"1.4\"\n  ]",
            "(defun f (a b) a)",
            "(macrolet ((f (x) (f x 0)))",
            "(f 1))",
            "(f 2 0))",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_all_calls_respects_common_lisp_cl_user_flet_shadowing() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "b",
            "--argument",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "\
(defun f (a) a)
(defun caller ()
  (cl-user:flet ((f (x) (f x)))
    (f 1))
  (f 2))",
        &[
            "\"all_calls\": true",
            "\"call_paths\": [\n    \"1.3.1.0.2\",\n    \"1.4\"\n  ]",
            "(defun f (a b) a)",
            "(cl-user:flet ((f (x) (f x 0)))",
            "(f 1))",
            "(f 2 0))",
        ],
    );
}

#[test]
fn cli_plans_add_function_parameter_all_calls_respects_common_lisp_cl_user_macrolet_shadowing() {
    assert_add_function_parameter_success(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "b",
            "--argument",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "\
(defun f (a) a)
(defun caller ()
  (cl-user:macrolet ((f (x) (f x)))
    (f 1))
  (f 2))",
        &[
            "\"all_calls\": true",
            "\"call_paths\": [\n    \"1.3.1.0.2\",\n    \"1.4\"\n  ]",
            "(defun f (a b) a)",
            "(cl-user:macrolet ((f (x) (f x 0)))",
            "(f 1))",
            "(f 2 0))",
        ],
    );
}

#[test]
fn cli_rejects_add_function_parameter_explicit_labels_shadowed_call_path() {
    assert_add_function_parameter_failure(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "b",
            "--argument",
            "0",
            "--call-path",
            "1.3.1.0.2",
        ],
        "\
(defun f (a) a)
(defun caller ()
  (labels ((f (x) (f x))
           (g (y) (f y)))
    (f 1)
    (cl:print (f 2)))
  (f 3))",
        &["shadowed by a local callable binding or overlaps the selected definition"],
    );
}

#[test]
fn cli_rejects_add_function_parameter_explicit_macrolet_shadowed_call_path() {
    assert_add_function_parameter_failure(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "b",
            "--argument",
            "0",
            "--call-path",
            "1.3.2",
        ],
        "\
(defun f (a) a)
(defun caller ()
  (macrolet ((f (x) (f x)))
    (f 1))
  (f 2))",
        &["shadowed by a local callable binding or overlaps the selected definition"],
    );
}

#[test]
fn cli_rejects_add_function_parameter_explicit_compiler_macrolet_shadowed_call_path() {
    assert_add_function_parameter_failure(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "b",
            "--argument",
            "0",
            "--call-path",
            "1.3.2",
        ],
        "\
(defun f (a) a)
(defun caller ()
  (compiler-macrolet ((f (x) (f x)))
    (f 1))
  (f 2))",
        &["shadowed by a local callable binding or overlaps the selected definition"],
    );
}

#[test]
fn cli_rejects_add_function_parameter_explicit_cl_user_compiler_macrolet_shadowed_call_path() {
    assert_add_function_parameter_failure(
        &[
            "add-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "b",
            "--argument",
            "0",
            "--call-path",
            "1.3.2",
        ],
        "\
(defun f (a) a)
(defun caller ()
  (cl-user:compiler-macrolet ((f (x) (f x)))
    (f 1))
  (f 2))",
        &["shadowed by a local callable binding or overlaps the selected definition"],
    );
}
