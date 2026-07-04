use super::*;

#[test]
fn cli_rejects_function_parameter_all_calls_with_explicit_call_path() {
    let mut cmd = paredit();
    cmd.args([
        "remove-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--all-calls",
        "--call-path",
        "1.3",
    ])
    .write_stdin(
        "(defun area (width height margin) (* width height))\n(defun render () (area 10 20 5))",
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "either --all-calls or repeated --call-path",
    ));
}
