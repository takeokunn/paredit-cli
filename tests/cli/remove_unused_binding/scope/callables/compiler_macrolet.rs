use super::*;

#[test]
fn cli_plans_remove_unused_compiler_macrolet_without_counting_expander_body_reference() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "value",
        "--output",
        "json",
    ])
    .write_stdin(
        "(compiler-macrolet ((value (x) (compute value x)) (used (y) (list y))) (list used))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"compiler-macrolet\""))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(x) (compute value x)\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(compiler-macrolet ((used (y)\\n                      (list y)))\\n  (list used))",
    ));
}

#[test]
fn cli_plans_remove_unused_cl_compiler_macrolet_without_counting_expander_body_reference() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "value",
        "--output",
        "json",
    ])
    .write_stdin(
        "(cl:compiler-macrolet ((value (x) (compute value x)) (used (y) (list y))) (list used))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"form\": \"cl:compiler-macrolet\"",
    ))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(x) (compute value x)\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(cl:compiler-macrolet ((used (y)\\n                         (list y)))\\n  (list used))",
    ));
}

#[test]
fn cli_plans_remove_unused_cl_user_compiler_macrolet_without_counting_expander_body_reference() {
    let mut cmd = paredit();
    cmd.args(["refactor", "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "value",
        "--output",
        "json",
    ])
    .write_stdin(
        "(cl-user:compiler-macrolet ((value (x) (compute value x)) (used (y) (list y))) (list used))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"cl-user:compiler-macrolet\""))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(x) (compute value x)\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(cl-user:compiler-macrolet ((used (y)\\n                              (list y)))\\n  (list used))",
    ));
}

#[test]
fn cli_rejects_referenced_compiler_macrolet_binding() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "value",
    ])
    .write_stdin("(compiler-macrolet ((value (x) (compute x))) (list (value 1)))")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "remove-unused-binding requires zero in-scope references",
    ));
}
