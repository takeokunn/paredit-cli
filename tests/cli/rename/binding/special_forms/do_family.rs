use super::*;

#[test]
fn cli_plans_do_binding_rename_across_steps_end_clause_and_body() {
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
    .write_stdin("(do ((value seed (1+ value))) ((done value) value) (collect value seed))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"do\""))
    .stdout(predicate::str::contains("\"reference_count\": 4"))
    .stdout(predicate::str::contains(
        "(do ((item seed (1+ item))) ((done item) item) (collect item seed))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_without_touching_do_scope() {
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
        "(let ((value 1)) (list value (do ((value value (1+ value))) ((done) value) value) value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 3"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (list outer (do ((value outer (1+ value))) ((done) value) value) outer))",
    ));
}

#[test]
fn cli_plans_do_star_binding_rename_across_later_inits_steps_end_clause_and_body() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
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
    .write_stdin(
        "(do* ((value seed (1+ value)) (copy value)) ((done value) (list value copy)) (collect value copy))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"do*\""))
    .stdout(predicate::str::contains("\"reference_count\": 5"))
    .stdout(predicate::str::contains(
        "(do* ((item seed (1+ item)) (copy item)) ((done item) (list item copy)) (collect item copy))",
    ));
}
