use super::*;

#[test]
fn cli_plans_prog_star_binding_rename_across_later_inits_and_body() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
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
    .write_stdin("(prog* ((value seed) (copy value)) (return (list value copy)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"prog*\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(prog* ((item seed) (copy item)) (return (list item copy)))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_without_touching_prog_scope() {
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
    .write_stdin(
        "(let ((value 1)) (list value (prog ((value value) (copy value)) (return value)) value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 4"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (list outer (prog ((value outer) (copy outer)) (return value)) outer)",
    ));
}
