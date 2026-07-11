use super::*;

#[test]
fn cli_plans_remove_unused_macrolet_without_counting_expander_body_reference() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "value",
        "--output",
        "json",
    ])
    .write_stdin("(macrolet ((value (x) (compute value x)) (used (y) (list y))) (list used))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"macrolet\""))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(x) (compute value x)\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(macrolet ((used (y)\\n             (list y)))\\n  (list used))",
    ));
}

#[test]
fn cli_plans_remove_unused_cl_macrolet_without_counting_expander_body_reference() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "value",
        "--output",
        "json",
    ])
    .write_stdin("(cl:macrolet ((value (x) (compute value x)) (used (y) (list y))) (list used))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"cl:macrolet\""))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(x) (compute value x)\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(cl:macrolet ((used (y)\\n                (list y)))\\n  (list used))",
    ));
}

#[test]
fn cli_plans_remove_unused_cl_user_macrolet_without_counting_expander_body_reference() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "value",
        "--output",
        "json",
    ])
    .write_stdin(
        "(cl-user:macrolet ((value (x) (compute value x)) (used (y) (list y))) (list used))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"cl-user:macrolet\""))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(x) (compute value x)\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(cl-user:macrolet ((used (y)\\n                     (list y)))\\n  (list used))",
    ));
}

#[test]
fn cli_rejects_referenced_macrolet_binding() {
    let mut cmd = paredit();
    cmd.args(["remove-unused-binding", "--path", "0", "--name", "value"])
        .write_stdin("(macrolet ((value (x) (compute x))) (list (value 1)))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "remove-unused-binding requires zero in-scope references",
        ));
}

#[test]
fn cli_plans_remove_unused_compiler_macrolet_without_counting_expander_body_reference() {
    let mut cmd = paredit();
    cmd.args([
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
    cmd.args([
        "remove-unused-binding",
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
    cmd.args(["remove-unused-binding", "--path", "0", "--name", "value"])
        .write_stdin("(compiler-macrolet ((value (x) (compute x))) (list (value 1)))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "remove-unused-binding requires zero in-scope references",
        ));
}

#[test]
fn cli_plans_remove_unused_flet_binding_ignoring_definition_body_reference() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
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
    cmd.args(["remove-unused-binding", "--path", "0", "--name", "unused"])
        .write_stdin("(labels ((unused () (unused)) (used () (list used))) (used))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "remove-unused-binding requires zero in-scope references",
        ));
}
