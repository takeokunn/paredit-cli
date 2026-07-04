use super::*;

#[test]
fn cli_plans_binding_rename_without_shadowed_inner_binding() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
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
        "rename-binding",
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
fn cli_plans_lambda_parameter_rename_without_shadow_capture() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(lambda (value) (list value (lambda (value) value) value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"lambda\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(lambda (product) (list product (lambda (value) value) product))",
    ));
}

#[test]
fn cli_plans_defun_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (value other) (list value other))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defun\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defun render (product other) (list product other))",
    ));
}

#[test]
fn cli_plans_define_setf_expander_environment_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "env",
        "--to",
        "macro-env",
        "--output",
        "json",
    ])
    .write_stdin(
        "(define-setf-expander slot (&whole whole &environment env target) (list whole env target))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"form\": \"define-setf-expander\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(define-setf-expander slot (&whole whole &environment macro-env target) (list whole macro-env target))",
    ));
}

#[test]
fn cli_plans_define_compiler_macro_environment_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "env",
        "--to",
        "macro-env",
        "--output",
        "json",
    ])
    .write_stdin(
        "(define-compiler-macro render (&whole whole &environment env target) (list whole env target))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"form\": \"define-compiler-macro\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(define-compiler-macro render (&whole whole &environment macro-env target) (list whole macro-env target))",
    ));
}
