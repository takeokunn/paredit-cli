use super::*;

#[test]
fn cli_rejects_reorder_function_parameters_when_call_argument_is_missing() {
    let mut cmd = reorder_command();
    cmd.args(["--definition-path", "0", "--call-path", "1.3"])
        .args(["--parameter", "height", "--parameter", "width"])
        .write_stdin("(defun area (width height) (* width height))\n(defun render () (area 10))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("has 1 arguments"));
}

#[test]
fn cli_rejects_reorder_function_parameters_across_common_lisp_lambda_list_sections() {
    let mut cmd = reorder_command();
    cmd.args(common_lisp_reorder_args("0", "1.3"))
        .args(["--parameter", "width", "--parameter", "node", "--parameter", "height"])
        .write_stdin(
            "(defun render (node &key width height) (list node width height))\n(defun draw () (render widget :width 10 :height 20))",
        )
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot move 'width' across Common Lisp lambda-list sections",
        ));
}
