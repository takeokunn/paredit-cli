use super::*;

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
fn cli_plans_remove_unused_emacs_lisp_cl_symbol_macrolet_without_counting_expansion_reference() {
    let dir = fresh_temp_dir("remove-unused-binding");
    let elisp_file = dir.join("render.el");
    fs::write(
        &elisp_file,
        "(cl-symbol-macrolet ((value (compute value)) (used other)) (list used))\n",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-binding")
        .arg("--file")
        .arg(&elisp_file)
        .arg("--path")
        .arg("0")
        .arg("--name")
        .arg("value")
        .arg("--allow-drop-value")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"binding_name\": \"value\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(elisp_file).expect("read remove unused binding elisp"),
        "(cl-symbol-macrolet ((used other))\n  (list used))\n"
    );
}

#[test]
fn cli_plans_remove_unused_cl_user_symbol_macrolet_without_counting_expansion_reference() {
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
    .write_stdin("(cl-user:symbol-macrolet ((value (compute value)) (used other)) (list used))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"form\": \"cl-user:symbol-macrolet\"",
    ))
    .stdout(predicate::str::contains("\"binding_name\": \"value\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(compute value)\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 0"))
    .stdout(predicate::str::contains(
        "(cl-user:symbol-macrolet ((used other))\\n  (list used))",
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
