use super::*;

#[test]
fn cli_plans_symbol_macrolet_binding_rename_without_touching_expansion_reference() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
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
fn cli_plans_binding_rename_inside_quasiquote_preserving_unquote_prefixes() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "forms",
        "--output",
        "json",
    ])
    .write_stdin("(let ((value items)) `(,value ,@value value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(let ((forms items)) `(,forms ,@forms value))",
    ));
}

#[test]
fn cli_plans_binding_rename_only_after_matching_nested_unquote_depth() {
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
    .write_stdin("(let ((value 1)) `(outer `,value ,,value value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((item 1)) `(outer `,value ,,item value))",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_through_symbol_macrolet_expansion_only() {
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

#[test]
fn cli_plans_outer_binding_rename_through_macrolet_expander_only() {
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
