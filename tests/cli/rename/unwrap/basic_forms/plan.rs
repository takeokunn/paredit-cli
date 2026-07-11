use super::*;

#[test]
fn cli_plans_unwrap_function_calls_without_writing() {
    let dir = fresh_temp_dir("unwrap-function-calls-plan");
    let lisp_file = dir.join("service.lisp");
    let source =
        "(defun render ()\n  (with-cache (fetch-user id))\n  (with-cache (fetch-user other)))\n";
    fs::write(&lisp_file, source).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("unwrap-function-calls")
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
