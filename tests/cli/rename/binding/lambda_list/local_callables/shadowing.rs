use super::super::*;

#[test]
fn cli_plans_outer_binding_rename_without_touching_local_callable_lambda_shadow() {
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
        "item",
        "--output",
        "json",
    ])
    .write_stdin("(let ((value 1)) (flet ((helper (value) value)) (helper value) value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((item 1)) (flet ((helper (value) value)) (helper item) item))",
    ));
}
