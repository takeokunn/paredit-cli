use super::*;

#[test]
fn cli_plans_binding_rename_inside_quasiquote_preserving_unquote_prefixes() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
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
        "refactor",
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
