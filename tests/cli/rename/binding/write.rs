use super::*;

#[test]
fn cli_writes_binding_rename_without_touching_shadowed_scope() {
    let dir = fresh_temp_dir("rename-binding");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (let ((value 1)) (+ value (let ((value 2)) value) value)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-binding")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--from")
        .arg("value")
        .arg("--to")
        .arg("product")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun render () (let ((product 1)) (+ product (let ((value 2)) value) product)))\n"
    );
}

#[test]
fn cli_rejects_missing_binding_rename_target() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "missing",
        "--to",
        "renamed",
    ])
    .write_stdin("(let ((value 1)) value)")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "binding 'missing' was not found in selected let",
    ));
}
