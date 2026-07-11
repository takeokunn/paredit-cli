use super::*;

#[test]
fn cli_rejects_wrap_function_calls_without_explicit_scope() {
    let dir = fresh_temp_dir("wrap-function-calls-no-scope");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () (fetch-user id))\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "wrap-function-calls requires either --all-calls or repeated --call-path",
        ));
}

#[test]
fn cli_rejects_wrap_function_calls_with_conflicting_scope_flags() {
    let dir = fresh_temp_dir("wrap-function-calls-conflicting-scope");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () (fetch-user id))\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--all-calls")
        .arg("--call-path")
        .arg("0.4")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "wrap-function-calls requires either --all-calls or repeated --call-path",
        ));
}
