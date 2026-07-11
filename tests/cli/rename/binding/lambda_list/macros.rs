use super::*;

#[test]
fn cli_plans_defmacro_environment_parameter_rename_without_touching_body() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
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
    .write_stdin("(defmacro inspect (&environment env value) (list env value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmacro\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmacro inspect (&environment macro-env value) (list macro-env value))",
    ));
}

#[test]
fn cli_plans_defmacro_whole_and_environment_parameter_rename_without_touching_body() {
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
        "(defmacro inspect (&whole whole &environment env value) (list whole env value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmacro\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmacro inspect (&whole whole &environment macro-env value) (list whole macro-env value))",
    ));
}

#[test]
fn cli_plans_defmacro_aux_parameter_rename_without_touching_aux_initializer() {
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
    .write_stdin("(defmacro inspect (&whole form value &aux (tag form)) (list form value tag))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmacro\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(defmacro inspect (&whole macro-form value &aux (tag macro-form)) (list macro-form value tag))",
    ));
}

#[test]
fn cli_plans_defmacro_optional_parameter_rename_without_touching_default_form() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-binding",
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
        "(defmacro wrap (&optional (value (default value) supplied)) (list value supplied))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmacro\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmacro wrap (&optional (form (default value) supplied)) (list form supplied))",
    ));
}

#[test]
fn cli_plans_define_setf_expander_environment_parameter_rename() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
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
        "(define-setf-expander slot (&whole whole &environment env target) (list whole env target))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"form\": \"define-setf-expander\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(define-setf-expander slot (&whole whole &environment macro-env target) (list whole macro-env target))",
    ));
}

#[test]
fn cli_plans_define_compiler_macro_environment_parameter_rename() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-binding",
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
        "(define-compiler-macro render (&whole whole &environment env target) (list whole env target))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"form\": \"define-compiler-macro\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(define-compiler-macro render (&whole whole &environment macro-env target) (list whole macro-env target))",
    ));
}
