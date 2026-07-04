use super::*;

#[test]
fn cli_plans_replace_function_calls_without_writing() {
    let dir = fresh_temp_dir("replace-function-calls-plan");
    let lisp_file = dir.join("service.lisp");
    let source = "(defun fetch-user (id) (list fetch-user id))\n(defun render () (fetch-user id) (fetch-user other))\n";
    fs::write(&lisp_file, source).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--all-calls")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"from\": \"fetch-user\""))
        .stdout(predicate::str::contains("\"to\": \"load-user\""))
        .stdout(predicate::str::contains("\"callCount\": 2"))
        .stdout(predicate::str::contains(
            "(defun fetch-user (id) (list fetch-user id))",
        ))
        .stdout(predicate::str::contains("(load-user id)"))
        .stdout(predicate::str::contains("(load-user other)"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read unchanged lisp"),
        source
    );
}

#[test]
fn cli_writes_replace_function_calls_at_call_path() {
    let dir = fresh_temp_dir("replace-function-calls-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (fetch-user id) (fetch-user other))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--call-path")
        .arg("0.4")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"path\": \"0.4\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read path-targeted lisp"),
        "(defun render () (fetch-user id) (load-user other))\n"
    );
}

#[test]
fn cli_rejects_replace_function_calls_without_explicit_scope() {
    let dir = fresh_temp_dir("replace-function-calls-no-scope");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () (fetch-user id))\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "replace-function-calls requires either --all-calls or repeated --call-path",
        ));
}

#[test]
fn cli_replace_function_calls_enforces_required_call_count() {
    let dir = fresh_temp_dir("replace-function-calls-require");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () (fetch-user id))\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--all-calls")
        .arg("--require-calls")
        .arg("2")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "expected at least 2 changed call sites but found 1",
        ));
}
