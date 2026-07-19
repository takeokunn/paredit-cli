use super::*;

#[test]
fn cli_plans_loop_for_in_binding_rename_without_touching_source() {
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
    .write_stdin("(loop for value in values collect value finally (return value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"loop\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(loop for item in values collect item finally (return item))",
    ));
}

#[test]
fn cli_plans_loop_with_binding_rename_without_touching_init() {
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
        "acc",
        "--output",
        "json",
    ])
    .write_stdin("(loop with value = (seed value) collect value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"loop\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(loop with acc = (seed value) collect acc)",
    ));
}

#[test]
fn cli_plans_loop_destructuring_binding_rename_without_touching_source() {
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
        "item-value",
        "--output",
        "json",
    ])
    .write_stdin("(loop for (key value) in pairs collect (list key value pairs))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"loop\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(loop for (key item-value) in pairs collect (list key item-value pairs))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_without_touching_loop_shadow() {
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
        "outer",
        "--output",
        "json",
    ])
    .write_stdin("(let ((value 1)) (loop for value in values collect value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (loop for value in values collect value) outer)",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_without_touching_loop_destructuring_shadow() {
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
        "outer",
        "--output",
        "json",
    ])
    .write_stdin("(let ((value 1)) (loop for (key value) in pairs collect value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (loop for (key value) in pairs collect value) outer)",
    ));
}
