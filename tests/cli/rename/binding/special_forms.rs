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

#[test]
fn cli_plans_do_binding_rename_across_steps_end_clause_and_body() {
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

#[test]
fn cli_plans_prog_star_binding_rename_across_later_inits_and_body() {
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
fn cli_plans_outer_binding_rename_without_touching_prog_scope() {
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

#[test]
fn cli_plans_loop_for_in_binding_rename_without_touching_source() {
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
        "rename-binding",
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
        "rename-binding",
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
    .write_stdin("(let ((value 1)) (loop for (key value) in pairs collect value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (loop for (key value) in pairs collect value) outer)",
    ));
}
