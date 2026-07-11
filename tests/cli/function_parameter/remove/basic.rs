pub(super) use super::*;

#[test]
fn cli_plans_remove_function_parameter_for_common_lisp() {
    let report = assert_remove_success_output(
        &[
            "remove-function-parameter",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--call-path",
            "1.3",
            "--output",
            "json",
        ],
        "(defun area (width height margin) (* width height))\n(defun render () (area 10 20 5))",
    );

    assert_eq!(report.function_name, "area");
    assert_eq!(report.parameter_name, "margin");
    assert_eq!(report.parameter_index, 2);
    assert_eq!(report.parameter_keyword, None);
    assert_eq!(report.removed_arguments, vec![Some("5".to_owned())]);
    assert!(report.changed);
    assert!(!report.written);
    assert_eq!(
        report.rewritten,
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))"
    );
}
