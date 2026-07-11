use super::super::*;

#[test]
fn cli_plans_flet_lambda_list_parameter_rename_without_touching_call_args() {
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
    .write_stdin("(flet ((helper (value) (list value))) (helper value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"flet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(flet ((helper (item) (list item))) (helper value))",
    ));
}

#[test]
fn cli_rejects_ambiguous_local_callable_lambda_list_parameter_rename() {
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
    .write_stdin("(flet ((left (value) value) (right (value) value)) (left 1))")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "multiple selected flet local callable lambda lists",
    ));
}
