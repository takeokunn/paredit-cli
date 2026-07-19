use super::*;

#[test]
fn cli_plans_binding_rename_without_shadowed_inner_binding() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0.3",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (let ((value 1)) (+ value (let ((value 2)) value) value)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"binding_span\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product 1)) (+ product (let ((value 2)) value) product)))",
    ));
}

#[test]
fn cli_plans_let_star_binding_rename_through_later_binding_values() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "seed",
        "--output",
        "json",
    ])
    .write_stdin("(let* ((value 1) (next (+ value 1))) (+ next value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let*\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(let* ((seed 1) (next (+ seed 1))) (+ next seed))",
    ));
}

#[test]
fn cli_plans_emacs_lisp_let_binding_rename_without_shadowed_inner_binding() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "emacs-lisp",
        "--path",
        "0.3",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (let ((value 1)) (+ value (let ((value 2)) value) value)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"binding_span\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product 1)) (+ product (let ((value 2)) value) product)))",
    ));
}

#[test]
fn cli_plans_emacs_lisp_let_star_binding_rename_through_later_binding_values() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "emacs-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "seed",
        "--output",
        "json",
    ])
    .write_stdin("(let* ((value 1) (next (+ value 1))) (+ next value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let*\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(let* ((seed 1) (next (+ seed 1))) (+ next seed))",
    ));
}

#[test]
fn cli_plans_common_lisp_bare_let_binding_rename_without_touching_later_init() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "seed",
        "--output",
        "json",
    ])
    .write_stdin("(let (value (next value)) (list value next))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(let (seed (next value)) (list seed next))",
    ));
}

#[test]
fn cli_plans_common_lisp_bare_let_star_binding_rename_through_later_init() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "seed",
        "--output",
        "json",
    ])
    .write_stdin("(let* (value (next value)) (list value next))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let*\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(let* (seed (next seed)) (list seed next))",
    ));
}

#[test]
fn cli_plans_outer_let_binding_rename_without_touching_inner_bare_binding() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "seed",
        "--output",
        "json",
    ])
    .write_stdin("(let ((value 1)) (let (value) value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((seed 1)) (let (value) value) seed)",
    ));
}
