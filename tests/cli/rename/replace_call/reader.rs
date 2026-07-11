use super::*;

#[test]
fn cli_replaces_only_calls_in_executable_reader_contexts() {
    let dir = fresh_temp_dir("replace-function-calls-reader-context");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () '(fetch-user quoted) `(list (fetch-user data) ,(fetch-user active)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--all-calls")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun render () '(fetch-user quoted) `(list (fetch-user data) ,(load-user active)))\n"
    );
}

#[test]
fn cli_rejects_replace_call_path_in_non_executable_reader_context() {
    let dir = fresh_temp_dir("replace-function-calls-reader-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () '(fetch-user quoted))\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--call-path")
        .arg("0.3")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "call-path 0.3 is not in an executable reader context",
        ));
}
