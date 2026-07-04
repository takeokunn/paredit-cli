use super::*;

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
        "(defun render () (let ((kept 3))\n  kept))\n"
    );
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
