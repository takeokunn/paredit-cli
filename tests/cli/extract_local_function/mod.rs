use super::*;

#[test]
fn cli_plans_extract_local_function_with_inferred_params() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "extract-local-function",
        "--dialect",
        "common-lisp",
        "--path",
        "0.3.1",
        "--enclosing-path",
        "0.3",
        "--name",
        "compute",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (x) (print (+ x 1)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"params\": [\n    \"x\""))
    .stdout(predicate::str::contains("\"binding\": \"flet\""))
    .stdout(predicate::str::contains(
        "(defun render (x) (flet ((compute (x) (+ x 1))) (print (compute x))))",
    ));
}

#[test]
fn cli_uses_labels_for_recursive_local_function() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "extract-local-function",
        "--dialect",
        "common-lisp",
        "--path",
        "0.3.1",
        "--enclosing-path",
        "0.3",
        "--name",
        "compute",
        "--recursive",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (print (+ 1 2)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"binding\": \"labels\""));
}
