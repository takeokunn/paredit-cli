use super::*;

#[test]
fn cli_plans_remove_unused_binding_without_writing() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0.3",
        "--name",
        "unused",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (let ((unused (compute)) (kept 1)) kept))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"binding_name\": \"unused\""))
    .stdout(predicate::str::contains("\"binding_value\": \"(compute)\""))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "\"dropped_value_requires_review\": true",
    ))
    .stdout(predicate::str::contains("\"written\": false"))
    .stdout(predicate::str::contains("(let ( (kept 1)) kept)"));
}

#[test]
fn cli_plans_remove_all_unused_bindings_without_writing() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0.3",
        "--all-bindings",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (let ((unused 1) (also 2) (kept 3)) kept))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"binding_count\": 2"))
    .stdout(predicate::str::contains("\"binding_name\": \"unused\""))
    .stdout(predicate::str::contains("\"binding_name\": \"also\""))
    .stdout(predicate::str::contains(
        "\"replacement\": \"(let (  (kept 3)) kept)\"",
    ))
    .stdout(predicate::str::contains("\"written\": false"));
}

#[test]
fn cli_plans_remove_all_unused_bindings_with_multiple_body_expressions() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0",
        "--all-bindings",
        "--output",
        "json",
    ])
    .write_stdin("(let ((unused 1) (also 2)) (log :start) :ok)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"binding_count\": 2"))
    .stdout(predicate::str::contains("\"binding_name\": \"unused\""))
    .stdout(predicate::str::contains("\"binding_name\": \"also\""))
    .stdout(predicate::str::contains(
        "\"replacement\": \"(log :start) :ok\"",
    ))
    .stdout(predicate::str::contains(
        "\"rewritten\": \"(log :start) :ok\"",
    ))
    .stdout(predicate::str::contains("\"written\": false"));
}

#[test]
fn cli_plans_remove_unused_binding_for_clojure_vector_binding() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--dialect",
        "clojure",
        "--path",
        "0",
        "--name",
        "unused",
        "--output",
        "json",
    ])
    .write_stdin("(let [unused 1 kept 2] kept)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"dialect\": \"clojure\""))
    .stdout(predicate::str::contains("\"binding_name\": \"unused\""))
    .stdout(predicate::str::contains("\"binding_value\": \"1\""))
    .stdout(predicate::str::contains("(let [ kept 2] kept)"));
}
