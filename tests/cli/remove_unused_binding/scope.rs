use super::*;

#[test]
fn cli_plans_remove_unused_binding_ignoring_shadowed_lambda_parameter() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0",
        "--name",
        "x",
        "--output",
        "json",
    ])
    .write_stdin("(let ((x 1) (used 2)) (list used (lambda (x) x)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"binding_name\": \"x\""))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(let ((used 2))\\n  (list\\n    used\\n    (lambda (x)\\n      x)))",
    ));
}

#[test]
fn cli_rejects_remove_unused_let_star_binding_used_later() {
    let mut cmd = paredit();
    cmd.args(["remove-unused-binding", "--path", "0", "--name", "x"])
        .write_stdin("(let* ((x 1) (y (+ x 2))) y)")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "remove-unused-binding requires zero in-scope references",
        ));
}

#[test]
fn cli_keeps_let_star_binding_used_by_later_binding_in_all_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "remove-unused-binding",
        "--path",
        "0",
        "--all-bindings",
        "--output",
        "json",
    ])
    .write_stdin("(let* ((x 1) (unused 2) (y (+ x 3))) y)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"binding_count\": 1"))
    .stdout(predicate::str::contains("\"binding_name\": \"unused\""))
    .stdout(predicate::str::contains(
        "\"replacement\": \"(let* ((x 1)\\n       (y (+ x 3)))\\n  y)\"",
    ));
}

#[test]
fn cli_plans_remove_unused_symbol_macrolet_without_counting_expansion_reference() {
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
    .write_stdin("(symbol-macrolet ((value (compute value)) (used other)) (list used))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"symbol-macrolet\""))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(compute value)\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(symbol-macrolet ((used other))\\n  (list used))",
    ));
}

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
        "(macrolet ((used (y) (list y)))\\n  (list used))",
    ));
}

#[test]
fn cli_rejects_referenced_symbol_macrolet_binding() {
    let mut cmd = paredit();
    cmd.args(["remove-unused-binding", "--path", "0", "--name", "value"])
        .write_stdin("(symbol-macrolet ((value (compute))) (list value))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "remove-unused-binding requires zero in-scope references",
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
fn cli_plans_remove_unused_with_slots_without_counting_instance_expression() {
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
    .write_stdin("(with-slots (value used) value (list used))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"with-slots\""))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains("\"binding_value\": \"value\""))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(with-slots (used)\\n  value\\n  (list used))",
    ));
}

#[test]
fn cli_plans_remove_unused_with_accessors_without_counting_instance_expression() {
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
    .write_stdin("(with-accessors ((value slot-name) (used used-slot)) value (list used))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"with-accessors\""))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains("\"binding_value\": \"slot-name\""))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(with-accessors ((used used-slot))\\n  value\\n  (list used))",
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
        "(compiler-macrolet ((used (y) (list y)))\\n  (list used))",
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
        "(flet ((used () (used)))\\n  (used))",
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
