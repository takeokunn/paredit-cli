use super::*;

#[test]
fn cli_rejects_rename_symbol_macro_without_matching_definition() {
    let dir = fresh_temp_dir("rename-symbol-macro-no-definition");
    let lisp_file = dir.join("core.lisp");
    write_fixture(
        &lisp_file,
        "(list old-name (setf old-name 1))\n",
        "failure fixture",
    );

    let mut cmd = paredit();
    cmd.arg("rename-symbol-macro")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rename-symbol-macro requires at least one matching define-symbol-macro definition",
        ));

    assert_eq!(
        read_fixture(&lisp_file, "unchanged failure fixture"),
        "(list old-name (setf old-name 1))\n"
    );
}

#[test]
fn cli_help_describes_rename_symbol_macro_contract() {
    let mut cmd = paredit();
    cmd.arg("rename-symbol-macro")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Plan or apply a Common Lisp define-symbol-macro binding and value-reference rename across explicit files",
        ))
        .stdout(predicate::str::contains(
            "while keeping expansion and lexical shadowing boundaries separate",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--from <FROM>"))
        .stdout(predicate::str::contains("--to <TO>"))
        .stdout(predicate::str::contains("--write"))
        .stdout(predicate::str::contains("--output <OUTPUT>"))
        .stdout(predicate::str::contains("<FILES>..."));
}
