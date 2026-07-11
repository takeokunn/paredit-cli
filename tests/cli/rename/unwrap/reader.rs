use super::*;

#[test]
fn cli_unwraps_only_calls_in_executable_reader_contexts() {
    let dir = fresh_temp_dir("unwrap-function-calls-reader-context");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () '(with-cache (fetch-user quoted)) `(list (with-cache (fetch-user data)) ,(with-cache (fetch-user active))))\n",
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
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun render () '(with-cache (fetch-user quoted)) `(list (with-cache (fetch-user data)) ,(fetch-user active)))\n"
    );
}

#[test]
fn cli_rejects_unwrap_call_path_in_non_executable_reader_context() {
    let dir = fresh_temp_dir("unwrap-function-calls-reader-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () '(with-cache (fetch-user quoted)))\n")
        .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--call-path")
        .arg("0.3")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "call-path 0.3 is not in an executable reader context",
        ));
}
