use super::super::*;

#[test]
fn cli_plans_compiler_macrolet_lambda_list_parameter_rename() {
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
        "form",
        "--output",
        "json",
    ])
    .write_stdin("(compiler-macrolet ((expand (value) (list value))) (expand value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"compiler-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(compiler-macrolet ((expand (form) (list form))) (expand value) value)",
    ));
}

#[test]
fn cli_plans_compiler_macrolet_environment_parameter_rename_without_touching_body() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "env",
        "--to",
        "macro-env",
        "--output",
        "json",
    ])
    .write_stdin(
        "(compiler-macrolet ((expand (&whole whole &environment env value) (list whole env value))) (expand value) env)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"compiler-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(compiler-macrolet ((expand (&whole whole &environment macro-env value) (list whole macro-env value))) (expand value) env)",
    ));
}

#[test]
fn cli_plans_compiler_macrolet_aux_parameter_rename_without_touching_aux_initializer() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
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
        "(compiler-macrolet ((inspect (&whole form value &aux (tag form)) (list form value tag))) (inspect value) form)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"compiler-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(compiler-macrolet ((inspect (&whole macro-form value &aux (tag macro-form)) (list macro-form value tag))) (inspect value) form)",
    ));
}

#[test]
fn cli_plans_compiler_macrolet_key_parameter_rename_without_touching_key_designator_or_call_site() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
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
        "(compiler-macrolet ((inspect (&key ((:value value) (default value) value-supplied)) (list value value-supplied))) (inspect :value value) value)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"compiler-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(compiler-macrolet ((inspect (&key ((:value form) (default value) value-supplied)) (list form value-supplied))) (inspect :value value) value)",
    ));
}
