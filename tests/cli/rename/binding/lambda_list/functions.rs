use super::*;

#[test]
fn cli_plans_lambda_parameter_rename_without_shadow_capture() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
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
        "refactor",
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
fn cli_plans_emacs_lisp_lambda_parameter_rename_without_shadow_capture() {
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
fn cli_plans_emacs_lisp_defun_parameter_rename() {
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
fn cli_plans_defmethod_specialized_parameter_rename_without_touching_specializer() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--path",
        "0",
        "--from",
        "node",
        "--to",
        "widget-node",
        "--output",
        "json",
    ])
    .write_stdin("(defmethod render ((node widget) stream) (list node stream widget))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmethod\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmethod render ((widget-node widget) stream) (list widget-node stream widget))",
    ));
}

#[test]
fn cli_plans_defmethod_qualifier_parameter_rename() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
        "--path",
        "0",
        "--from",
        "node",
        "--to",
        "widget-node",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defmethod render :around ((node widget) stream) (call-next-method) (list node stream))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmethod\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmethod render :around ((widget-node widget) stream) (call-next-method) (list widget-node stream))",
    ));
}

#[test]
fn cli_plans_cl_defmethod_optional_parameter_rename_without_touching_default_form() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
        "--path",
        "0",
        "--from",
        "stream",
        "--to",
        "out",
        "--output",
        "json",
    ])
    .write_stdin(
        "(cl-defmethod render ((node widget) &optional (stream (default-stream node) stream-p)) (list node stream stream-p))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"cl-defmethod\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(cl-defmethod render ((node widget) &optional (out (default-stream node) stream-p)) (list node out stream-p))",
    ));
}
