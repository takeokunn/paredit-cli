use super::*;

#[test]
fn cli_rejects_local_function_rename_without_matching_definition() {
    let fixture = write_local_function_fixture(
        "rename-local-function-missing-definition",
        "lisp",
        "(old-name 1)\n",
    );

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&fixture)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rename-local-function requires at least one matching local function definition",
        ));

    assert_eq!(
        fs::read_to_string(&fixture).expect("read unchanged missing-definition fixture"),
        "(old-name 1)\n"
    );
}

#[test]
fn cli_help_describes_rename_local_function_contract() {
    let mut cmd = paredit();
    cmd.arg("refactor").arg("rename-local-function")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Plan or apply a Common Lisp flet/labels local function binding and call-site rename across explicit files",
        ))
        .stdout(predicate::str::contains(
            "preserving the difference between non-recursive flet bodies and recursive labels bodies",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--from <FROM>"))
        .stdout(predicate::str::contains("--to <TO>"))
        .stdout(predicate::str::contains("--write"))
        .stdout(predicate::str::contains("--output <OUTPUT>"))
        .stdout(predicate::str::contains("<FILES>..."));
}
