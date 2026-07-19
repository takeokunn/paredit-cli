use super::*;

#[test]
fn cli_plans_remove_unused_flet_binding_ignoring_definition_body_reference() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-unused-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--name",
        "unused",
        "--output",
        "json",
    ])
    .write_stdin("(flet ((unused () (unused)) (used () (used))) (used))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"flet\""))
    .stdout(predicate::str::contains("\"binding_name\": \"unused\""))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(flet ((used ()\\n         (used)))\\n  (used))",
    ));
}

#[test]
fn cli_rejects_recursive_labels_binding() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-unused-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--name",
        "unused",
    ])
    .write_stdin("(labels ((unused () (unused)) (used () (list used))) (used))")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "remove-unused-binding requires zero in-scope references",
    ));
}
