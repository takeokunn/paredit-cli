use super::super::*;

#[test]
fn cli_plans_macrolet_lambda_list_parameter_rename_without_touching_expansion_site() {
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
        "form",
        "--output",
        "json",
    ])
    .write_stdin("(macrolet ((wrap (value) (list value))) (wrap value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(macrolet ((wrap (form) (list form))) (wrap value) value)",
    ));
}

#[test]
fn cli_plans_macrolet_whole_parameter_rename_without_touching_call_site_form() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "whole",
        "--to",
        "form",
        "--output",
        "json",
    ])
    .write_stdin("(macrolet ((wrap (&whole whole value) (list whole value))) (wrap value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(macrolet ((wrap (&whole form value) (list form value))) (wrap value))",
    ));
}

#[test]
fn cli_plans_macrolet_aux_parameter_rename_without_touching_aux_initializer() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "form",
        "--to",
        "macro-form",
        "--output",
        "json",
    ])
    .write_stdin(
        "(macrolet ((inspect (&whole form value &aux (tag form)) (list form value tag))) (inspect value) form)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(macrolet ((inspect (&whole macro-form value &aux (tag macro-form)) (list macro-form value tag))) (inspect value) form)",
    ));
}

#[test]
fn cli_plans_macrolet_key_parameter_rename_without_touching_key_designator_or_call_site() {
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
        "form",
        "--output",
        "json",
    ])
    .write_stdin(
        "(macrolet ((inspect (&key ((:value value) (default value) value-supplied)) (list value value-supplied))) (inspect :value value) value)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(macrolet ((inspect (&key ((:value form) (default value) value-supplied)) (list form value-supplied))) (inspect :value value) value)",
    ));
}
