use super::*;

#[test]
fn cli_plans_outer_binding_rename_through_macrolet_expander_only() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "outer",
        "--output",
        "json",
    ])
    .write_stdin("(let ((value 1)) (list value (macrolet ((emit () value)) (emit) value) value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 4"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 0"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (list outer (macrolet ((emit () outer)) (emit) outer) outer))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_through_cl_user_macrolet_expander_only() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
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
        "(let ((value 1)) (list value (cl-user:macrolet ((emit () value)) (emit) value) value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 4"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 0"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (list outer (cl-user:macrolet ((emit () outer)) (emit) outer) outer))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_through_compiler_macrolet_expander_only() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
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
        "(let ((value 1)) (list value (compiler-macrolet ((emit () value)) (emit) value) value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 4"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 0"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (list outer (compiler-macrolet ((emit () outer)) (emit) outer) outer))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_through_cl_user_compiler_macrolet_expander_only() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
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
        "(let ((value 1)) (list value (cl-user:compiler-macrolet ((emit () value)) (emit) value) value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 4"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 0"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (list outer (cl-user:compiler-macrolet ((emit () outer)) (emit) outer) outer))",
    ));
}
