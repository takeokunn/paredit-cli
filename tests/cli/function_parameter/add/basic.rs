use super::*;

#[test]
fn cli_plans_add_function_parameter_for_common_lisp() {
    let output = run_add_function_parameter(
        &[
            "add-function-parameter",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--argument",
            "5",
            "--call-path",
            "1.3",
            "--output",
            "json",
        ],
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))",
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_add_function_parameter_report(&output.stdout).expect("parse add report");
    assert_eq!(report.function_name, "area");
    assert_eq!(report.parameter_name, "margin");
    assert_eq!(report.argument, "5");
    assert_eq!(report.insert, "end");
    assert_eq!(report.parameter_section, "required");
    assert!(report.changed);
    assert!(!report.written);
    assert_eq!(
        report.rewritten,
        "(defun area (width height margin) (* width height))\n(defun render () (area 10 20 5))"
    );
}

#[test]
fn cli_rejects_add_function_parameter_mismatched_call() {
    assert_add_function_parameter_failure(
        &[
            "add-function-parameter",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--argument",
            "5",
            "--call-path",
            "1.3",
        ],
        "(defun area (width height) (* width height))\n(defun render () (perimeter 10 20))",
        &["does not match selected definition"],
    );
}
