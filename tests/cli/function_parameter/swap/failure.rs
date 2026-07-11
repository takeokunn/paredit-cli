use super::*;

#[test]
fn cli_rejects_swap_function_parameters_when_call_argument_is_missing() {
    let mut cmd = swap_command();
    cmd.args([
        "--definition-path",
        "0",
        "--left-name",
        "width",
        "--right-name",
        "height",
        "--call-path",
        "1.3",
    ])
    .write_stdin("(defun area (width height) (* width height))\n(defun render () (area 10))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("has 1 arguments"));
}

#[test]
fn cli_rejects_swap_function_parameters_across_common_lisp_lambda_list_section() {
    let mut cmd = swap_command();
    cmd.args([
        "--dialect",
        "common-lisp",
        "--definition-path",
        "0",
        "--left-name",
        "scale",
        "--right-name",
        "width",
        "--call-path",
        "1.3",
    ])
    .write_stdin(
        "(defun area (scale &optional width) (* scale width))\n(defun render () (area 2 10))",
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "cannot move 'width' across Common Lisp lambda-list sections",
    ));
}
