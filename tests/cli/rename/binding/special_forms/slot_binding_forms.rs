use super::*;

#[test]
fn cli_plans_with_slots_binding_rename_preserving_slot_name() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "slot-value",
        "--output",
        "json",
    ])
    .write_stdin("(with-slots (value (alias slot-name)) object (list value alias object))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"with-slots\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(with-slots ((slot-value value) (alias slot-name)) object (list slot-value alias object))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_without_touching_with_slots_shadow() {
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
        "outer",
        "--output",
        "json",
    ])
    .write_stdin(
        "(let ((value 1)) (with-slots (value (alias value)) value (list value alias)) value)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (with-slots (value (alias value)) outer (list value alias)) outer)",
    ));
}

#[test]
fn cli_plans_with_accessors_binding_rename_preserving_accessor_name() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "slot-value",
        "--output",
        "json",
    ])
    .write_stdin(
        "(with-accessors ((value get-value) (alias get-alias)) object (list value alias object))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"with-accessors\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(with-accessors ((slot-value get-value) (alias get-alias)) object (list slot-value alias object))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_without_touching_with_accessors_shadow() {
    let mut cmd = paredit();
    cmd.args([
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
        "(let ((value 1)) (with-accessors ((value get-value) (alias value)) value (list value alias)) value)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (with-accessors ((value get-value) (alias value)) outer (list value alias)) outer)",
    ));
}

#[test]
fn cli_rejects_ambiguous_with_slots_binding_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "slot-value",
    ])
    .write_stdin("(with-slots (value (value slot-name)) object value)")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "binding 'value' was found in multiple selected with-slots specs",
    ));
}
