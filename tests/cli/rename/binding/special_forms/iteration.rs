use super::*;

#[test]
fn cli_plans_dolist_iteration_binding_rename_without_touching_source() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "item",
        "--output",
        "json",
    ])
    .write_stdin("(dolist (value items value) (collect value) items)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"dolist\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(dolist (item items item) (collect item) items)",
    ));
}

#[test]
fn cli_plans_dotimes_iteration_binding_rename_without_touching_count() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "index",
        "--to",
        "i",
        "--output",
        "json",
    ])
    .write_stdin("(dotimes (index limit index) (push index result) limit)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"dotimes\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(dotimes (i limit i) (push i result) limit)",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_without_touching_dolist_iteration_scope() {
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
        "seed",
        "--output",
        "json",
    ])
    .write_stdin(
        "(let ((value 1) (items 2)) (list value (dolist (value items value) value) value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((seed 1) (items 2)) (list seed (dolist (value items value) value) seed))",
    ));
}
