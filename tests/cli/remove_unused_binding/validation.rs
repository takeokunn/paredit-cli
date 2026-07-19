use super::*;

#[test]
fn cli_requires_file_for_remove_unused_binding_writes() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "unused",
        "--write",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_rejects_remove_unused_binding_for_unknown_stdin_dialect() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "unused",
    ])
    .write_stdin("(let ((unused 1)) :ok)")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "remove-unused-binding does not support dialect unknown",
    ));
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
    cmd.arg("refactor")
        .arg("remove-unused-binding")
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
    cmd.args([
        "refactor",
        "remove-unused-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--name",
        "x",
    ])
    .write_stdin("(let ((x 1)) (+ x 2))")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "remove-unused-binding requires zero in-scope references",
    ));
}

#[test]
fn cli_rejects_remove_unused_binding_name_with_all_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
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
    cmd.args([
        "refactor",
        "remove-unused-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--all-bindings",
    ])
    .write_stdin("(let ((x 1)) x)")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "remove-unused-binding --all-bindings found no unused bindings",
    ));
}
