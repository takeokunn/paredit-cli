use super::*;

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
