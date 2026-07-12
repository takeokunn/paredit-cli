use super::*;

#[test]
fn check_accepts_valid_input() {
    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("check")
        .write_stdin("(defun add (x y) (+ x y))")
        .assert()
        .success()
        .stdout("ok\n");
}

#[test]
fn check_rejects_invalid_input() {
    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("check")
        .write_stdin("(defun add (x y)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unclosed list"));
}

#[test]
fn check_rejects_local_function_used_as_a_value_from_standard_input() {
    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("check")
        .write_stdin("(flet ((finish-attempt () nil)) (funcall finish-attempt))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("function-used-as-value"));
}

#[test]
fn cli_selects_by_path() {
    let mut cmd = paredit();
    cmd.args(["edit", "select", "--path", "0.2"])
        .write_stdin("(defun add (x y) (+ x y))")
        .assert()
        .success()
        .stdout("(x y)");
}

#[test]
fn cli_replaces_by_path() {
    let mut cmd = paredit();
    cmd.args(["edit", "replace", "--path", "0.1", "--with", "sum"])
        .write_stdin("(defun add (x y) (+ x y))")
        .assert()
        .success()
        .stdout("(defun sum (x y) (+ x y))");
}

#[test]
fn cli_detects_emacs_lisp_from_extension() {
    let mut cmd = paredit();
    cmd.args(["inspect", "dialect", "--file", "tests/fixtures/sample.el"])
        .assert()
        .success()
        .stdout("emacs-lisp\n");
}

#[test]
fn cli_prints_definition_outline() {
    let mut cmd = paredit();
    cmd.args(["inspect", "outline", "--file", "tests/fixtures/sample.el"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0\t0..37\tdefun\ttrue"));
}

#[test]
fn cli_reports_selected_form_structure_for_agents() {
    let mut cmd = paredit();
    cmd.args([
        "inspect",
        "form",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--include-source",
        "--output",
        "json",
    ])
    .write_stdin("(defun add (x y) (+ x y))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
    .stdout(predicate::str::contains("\"path\": \"0\""))
    .stdout(predicate::str::contains("\"kind\": \"list\""))
    .stdout(predicate::str::contains("\"head\": \"defun\""))
    .stdout(predicate::str::contains("\"definitionLike\": true"))
    .stdout(predicate::str::contains("\"childCount\": 4"))
    .stdout(predicate::str::contains(
        "\"source\": \"(defun add (x y) (+ x y))\"",
    ))
    .stdout(predicate::str::contains("\"symbol\": \"x\""));
}

#[test]
fn cli_reports_form_by_byte_offset() {
    let mut cmd = paredit();
    cmd.args(["inspect", "form", "--at", "17", "--output", "json"])
        .write_stdin("(defun add (x y) (+ x y))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"path\": null"))
        .stdout(predicate::str::contains("\"head\": \"+\""))
        .stdout(predicate::str::contains("\"childCount\": 3"));
}
