use super::*;

#[test]
fn cli_rejects_move_function_parameter_target_out_of_bounds() {
    let mut cmd = move_command();
    cmd.args([
        "--definition-path",
        "0",
        "--name",
        "width",
        "--to-index",
        "2",
        "--call-path",
        "1.3",
    ])
    .write_stdin("(defun area (width height) (* width height))\n(defun render () (area 10 20))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("target index"));
}

#[test]
fn cli_rejects_move_function_parameter_across_common_lisp_lambda_list_section() {
    let mut cmd = move_command();
    cmd.args([
        "--dialect",
        "common-lisp",
        "--definition-path",
        "0",
        "--name",
        "width",
        "--to-index",
        "0",
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
