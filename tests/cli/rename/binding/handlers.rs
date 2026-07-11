use super::*;

#[test]
fn cli_plans_handler_bind_lambda_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "condition",
        "--to",
        "err",
        "--output",
        "json",
    ])
    .write_stdin(
        "(handler-bind ((error (lambda (condition) (recover condition outer)))) (log condition))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"handler-bind\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(handler-bind ((error (lambda (err) (recover err outer)))) (log condition))",
    ));
}

#[test]
fn cli_plans_restart_bind_lambda_parameter_rename() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "condition",
        "--to",
        "reason",
        "--output",
        "json",
    ])
    .write_stdin(
        "(restart-bind ((retry (lambda (condition) (recover condition)) :report (lambda (stream) stream))) (notify condition))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"restart-bind\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(restart-bind ((retry (lambda (reason) (recover reason)) :report (lambda (stream) stream))) (notify condition))",
    ));
}

#[test]
fn cli_rejects_ambiguous_handler_bind_lambda_parameter_rename() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "condition",
        "--to",
        "err",
        "--output",
        "json",
    ])
    .write_stdin(
        "(handler-bind ((error (lambda (condition) condition)) (warning (lambda (condition) condition))) (signal condition))",
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "multiple selected handler-bind handler functions",
    ));
}

#[test]
fn cli_plans_qualified_handler_bind_lambda_parameter_rename() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "condition",
        "--to",
        "err",
        "--output",
        "json",
    ])
    .write_stdin(
        "(cl-user:handler-bind ((error (lambda (condition) (recover condition outer)))) (log condition))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"cl-user:handler-bind\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(cl-user:handler-bind ((error (lambda (err) (recover err outer)))) (log condition))",
    ));
}

#[test]
fn cli_plans_qualified_handler_bind_qualified_lambda_parameter_rename() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "condition",
        "--to",
        "err",
        "--output",
        "json",
    ])
    .write_stdin(
        "(cl-user:handler-bind ((error (lambda (cl-user:condition) (recover condition outer)))) (log condition))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"cl-user:handler-bind\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(cl-user:handler-bind ((error (lambda (err) (recover err outer)))) (log condition))",
    ));
}

#[test]
fn cli_plans_qualified_restart_bind_lambda_parameter_rename() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "condition",
        "--to",
        "reason",
        "--output",
        "json",
    ])
    .write_stdin(
        "(cl-user:restart-bind ((retry (lambda (condition) (recover condition)) :report (lambda (stream) stream))) (notify condition))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"cl-user:restart-bind\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(cl-user:restart-bind ((retry (lambda (reason) (recover reason)) :report (lambda (stream) stream))) (notify condition))",
    ));
}
