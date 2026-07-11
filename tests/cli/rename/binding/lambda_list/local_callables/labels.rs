use super::super::*;

#[test]
fn cli_plans_labels_lambda_list_parameter_rename_without_touching_outer_body_call() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "node",
        "--output",
        "json",
    ])
    .write_stdin("(labels ((walk (value) (if value (walk value) value))) (walk seed) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"labels\""))
    .stdout(predicate::str::contains("\"reference_count\": 3"))
    .stdout(predicate::str::contains(
        "(labels ((walk (node) (if node (walk node) node))) (walk seed) value)",
    ));
}

#[test]
fn cli_rejects_ambiguous_labels_local_callable_lambda_list_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "node",
        "--output",
        "json",
    ])
    .write_stdin("(labels ((left (value) value) (right (value) value)) (left 1))")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "multiple selected labels local callable lambda lists",
    ));
}
