use super::*;

#[test]
fn cli_rejects_unwrap_function_calls_without_explicit_scope() {
    let dir = fresh_temp_dir("unwrap-function-calls-no-scope");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (with-cache (fetch-user id)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unwrap-function-calls requires either --all-calls or repeated --call-path",
        ));
}

#[test]
fn cli_rejects_unwrap_function_calls_with_conflicting_scope_flags() {
    let dir = fresh_temp_dir("unwrap-function-calls-conflicting-scope");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (with-cache (fetch-user id)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-function-calls")
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
            "unwrap-function-calls requires either --all-calls or repeated --call-path",
        ));
}

#[test]
fn cli_unwrap_function_calls_fails_when_selected_calls_do_not_change() {
    let dir = fresh_temp_dir("unwrap-function-calls-no-change");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (with-cache (fetch-user cached) :ttl 60))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--all-calls")
        .arg("--fail-on-no-change")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no selected call site changed"));
}
