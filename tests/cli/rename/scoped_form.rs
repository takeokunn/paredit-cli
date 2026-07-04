use super::*;

#[test]
fn cli_plans_scoped_rename_inside_selected_form() {
    let mut cmd = paredit();
    cmd.args([
        "rename-in-form",
        "--path",
        "0.3",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (let ((value 1)) (+ value other))) (defun value () value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"path\": \"0.3\""))
    .stdout(predicate::str::contains("\"from\": \"value\""))
    .stdout(predicate::str::contains("\"to\": \"product\""))
    .stdout(predicate::str::contains("\"count\": 2"))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product 1)) (+ product other))) (defun value () value)",
    ));
}

#[test]
fn cli_writes_scoped_rename_without_touching_other_forms() {
    let dir = fresh_temp_dir("rename-in-form");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (let ((value 1)) value))\n(defun value () value)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-in-form")
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
        "(defun render () (let ((product 1)) product))\n(defun value () value)\n"
    );
}

#[test]
fn cli_rejects_rename_in_form_write_without_file() {
    let mut cmd = paredit();
    cmd.args([
        "rename-in-form",
        "--path",
        "0.3",
        "--from",
        "value",
        "--to",
        "product",
        "--write",
    ])
    .write_stdin("(defun render () (let ((value 1)) value))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}
