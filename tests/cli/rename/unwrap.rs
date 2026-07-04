use super::*;

#[test]
fn cli_plans_unwrap_function_calls_without_writing() {
    let dir = fresh_temp_dir("unwrap-function-calls-plan");
    let lisp_file = dir.join("service.lisp");
    let source =
        "(defun render ()\n  (with-cache (fetch-user id))\n  (with-cache (fetch-user other)))\n";
    fs::write(&lisp_file, source).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-function-calls")
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
        .stdout(predicate::str::contains("(fetch-user id)"))
        .stdout(predicate::str::contains("(fetch-user other)"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read unchanged lisp"),
        source
    );
}

#[test]
fn cli_writes_unwrap_function_calls_and_skips_non_unary_wrappers() {
    let dir = fresh_temp_dir("unwrap-function-calls-write");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render ()\n  (with-cache (fetch-user id))\n  (with-cache (fetch-user cached) :ttl 60))\n",
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
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains(
            "\"skippedNonUnaryWrapperCount\": 1",
        ))
        .stdout(predicate::str::contains("\"written\": true"));

    let rewritten = fs::read_to_string(lisp_file).expect("read unwrapped lisp");
    assert!(rewritten.contains("(fetch-user id)"));
    assert!(rewritten.contains("(with-cache (fetch-user cached) :ttl 60)"));
    assert!(!rewritten.contains("(with-cache (fetch-user id))"));
}

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
fn cli_unwrap_function_calls_can_target_call_path() {
    let dir = fresh_temp_dir("unwrap-function-calls-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (with-cache (fetch-user id)) (with-cache (fetch-user other)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-function-calls")
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
        "(defun render () (with-cache (fetch-user id)) (fetch-user other))\n"
    );
}

#[test]
fn cli_unwrap_function_calls_skips_nested_all_call_rewrites() {
    let dir = fresh_temp_dir("unwrap-function-calls-nested");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (with-cache (fetch-user (with-cache (fetch-user id)))))\n",
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
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"skippedNestedCount\": 1"))
        .stdout(predicate::str::contains(
            "(fetch-user (with-cache (fetch-user id)))",
        ));
}
