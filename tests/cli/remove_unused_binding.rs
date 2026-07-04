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
fn cli_writes_remove_unused_single_binding_when_drop_value_allowed() {
    let dir = fresh_temp_dir("remove-unused-binding");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (let ((unused (compute))) :ok))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-binding")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--name")
        .arg("unused")
        .arg("--allow-drop-value")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"binding_name\": \"unused\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun render () :ok)\n"
    );
}

#[test]
fn cli_writes_remove_all_unused_bindings_when_drop_value_allowed() {
    let dir = fresh_temp_dir("remove-all-unused-bindings");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (let ((unused 1) (also 2) (kept 3)) kept))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-binding")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--all-bindings")
        .arg("--allow-drop-value")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"binding_count\": 2"))
        .stdout(predicate::str::contains("\"binding_name\": \"unused\""))
        .stdout(predicate::str::contains("\"binding_name\": \"also\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun render () (let (  (kept 3)) kept))\n"
    );
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
fn cli_writes_remove_all_unused_bindings_with_multiple_body_expressions() {
    let dir = fresh_temp_dir("remove-all-unused-bindings-multi-body");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (let ((unused 1) (also 2)) (log :start) :ok))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-binding")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--all-bindings")
        .arg("--allow-drop-value")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"binding_count\": 2"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun render () (log :start) :ok)\n"
    );
}

#[test]
fn cli_requires_drop_value_permission_for_remove_unused_binding_writes() {
    let dir = fresh_temp_dir("remove-unused-binding-permission");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (let ((unused (compute))) :ok))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-binding")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--name")
        .arg("unused")
        .arg("--write")
        .assert()
        .failure()
        .stderr(predicate::str::contains("pass --allow-drop-value"));
}

#[test]
fn cli_rejects_remove_unused_binding_with_references() {
    let mut cmd = paredit();
    cmd.args(["remove-unused-binding", "--path", "0", "--name", "x"])
        .write_stdin("(let ((x 1)) (+ x 2))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "remove-unused-binding requires zero in-scope references",
        ));
}

#[test]
fn cli_plans_remove_unused_binding_ignoring_shadowed_lambda_parameter() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "x",
        "--output",
        "json",
    ])
    .write_stdin("(let ((x 1) (used 2)) (list used (lambda (x) x)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"binding_name\": \"x\""))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(let ( (used 2)) (list used (lambda (x) x)))",
    ));
}

#[test]
fn cli_rejects_remove_unused_binding_name_with_all_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "x",
        "--all-bindings",
    ])
    .write_stdin("(let ((x 1)) :ok)")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "remove-unused-binding accepts either --name or --all-bindings",
    ));
}

#[test]
fn cli_rejects_remove_all_unused_bindings_when_none_unused() {
    let mut cmd = paredit();
    cmd.args(["remove-unused-binding", "--path", "0", "--all-bindings"])
        .write_stdin("(let ((x 1)) x)")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "remove-unused-binding --all-bindings found no unused bindings",
        ));
}

#[test]
fn cli_rejects_remove_unused_let_star_binding_used_later() {
    let mut cmd = paredit();
    cmd.args(["remove-unused-binding", "--path", "0", "--name", "x"])
        .write_stdin("(let* ((x 1) (y (+ x 2))) y)")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "remove-unused-binding requires zero in-scope references",
        ));
}

#[test]
fn cli_keeps_let_star_binding_used_by_later_binding_in_all_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0",
        "--all-bindings",
        "--output",
        "json",
    ])
    .write_stdin("(let* ((x 1) (unused 2) (y (+ x 3))) y)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"binding_count\": 1"))
    .stdout(predicate::str::contains("\"binding_name\": \"unused\""))
    .stdout(predicate::str::contains(
        "\"replacement\": \"(let* ((x 1)  (y (+ x 3))) y)\"",
    ));
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
