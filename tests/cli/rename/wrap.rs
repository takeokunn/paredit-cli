use super::*;

#[test]
fn cli_plans_wrap_function_calls_without_writing() {
    let dir = fresh_temp_dir("wrap-function-calls-plan");
    let lisp_file = dir.join("service.lisp");
    let source = "(defun render ()\n  (format-message (fetch-user id))\n  (fetch-user other))\n";
    fs::write(&lisp_file, source).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--all-calls")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"function\": \"fetch-user\""))
        .stdout(predicate::str::contains("\"wrapper\": \"with-cache\""))
        .stdout(predicate::str::contains("\"callCount\": 2"))
        .stdout(predicate::str::contains("(with-cache (fetch-user id))"))
        .stdout(predicate::str::contains("(with-cache (fetch-user other))"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read unchanged lisp"),
        source
    );
}

#[test]
fn cli_writes_wrap_function_calls_and_skips_already_wrapped() {
    let dir = fresh_temp_dir("wrap-function-calls-write");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render ()\n  (fetch-user id)\n  (with-cache (fetch-user cached)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
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
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains(
            "\"skippedAlreadyWrappedCount\": 1",
        ))
        .stdout(predicate::str::contains("\"written\": true"));

    let rewritten = fs::read_to_string(lisp_file).expect("read wrapped lisp");
    assert!(rewritten.contains("(with-cache (fetch-user id))"));
    assert!(rewritten.contains("(with-cache (fetch-user cached))"));
    assert!(!rewritten.contains("(with-cache (with-cache"));
}

#[test]
fn cli_rejects_wrap_function_calls_without_explicit_scope() {
    let dir = fresh_temp_dir("wrap-function-calls-no-scope");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () (fetch-user id))\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
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
fn cli_wrap_function_calls_can_target_call_path() {
    let dir = fresh_temp_dir("wrap-function-calls-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (fetch-user id) (fetch-user other))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--call-path")
        .arg("0.4")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"path\": \"0.4\""));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read path-targeted lisp"),
        "(defun render () (fetch-user id) (with-cache (fetch-user other)))\n"
    );
}

#[test]
fn cli_wrap_function_calls_skips_nested_all_call_rewrites() {
    let dir = fresh_temp_dir("wrap-function-calls-nested");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (fetch-user (fetch-user id)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--all-calls")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"skippedNestedCount\": 1"))
        .stdout(predicate::str::contains(
            "(with-cache (fetch-user (fetch-user id)))",
        ));
}
