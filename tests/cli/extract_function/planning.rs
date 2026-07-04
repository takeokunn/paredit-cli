use super::*;

#[test]
fn cli_plans_extract_function_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "compute-sum",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (+ 1 2))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"dialect\": \"unknown\""))
    .stdout(predicate::str::contains("\"call\": \"(compute-sum)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun compute-sum () (+ 1 2))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (compute-sum))\\n\\n(defun compute-sum () (+ 1 2))",
    ));
}
