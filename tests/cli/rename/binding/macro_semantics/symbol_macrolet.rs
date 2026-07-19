use super::*;

#[test]
fn cli_plans_symbol_macrolet_binding_rename_without_touching_expansion_reference() {
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
        "slot",
        "--output",
        "json",
    ])
    .write_stdin("(symbol-macrolet ((value (compute value))) (list value value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"symbol-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(symbol-macrolet ((slot (compute value))) (list slot slot))",
    ));
}

#[test]
fn cli_plans_emacs_lisp_cl_symbol_macrolet_binding_rename_without_touching_expansion_reference() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "emacs-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "slot",
        "--output",
        "json",
    ])
    .write_stdin("(cl-symbol-macrolet ((value (compute value))) (list value value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"cl-symbol-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(cl-symbol-macrolet ((slot (compute value))) (list slot slot))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_through_symbol_macrolet_expansion_only() {
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
        "(let ((value 1)) (list value (symbol-macrolet ((value (compute value))) value) value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 3"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (list outer (symbol-macrolet ((value (compute outer))) value) outer))",
    ));
}

#[test]
fn cli_plans_emacs_lisp_outer_binding_rename_through_cl_symbol_macrolet_expansion_only() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
        "--dialect",
        "emacs-lisp",
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
        "(let ((value 1)) (list value (cl-symbol-macrolet ((value (compute value))) value) value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 3"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (list outer (cl-symbol-macrolet ((value (compute outer))) value) outer))",
    ));
}
