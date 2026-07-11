use super::*;

#[cfg(unix)]
#[test]
fn cli_rolls_back_function_rename_when_later_file_write_fails() {
    assert_rollback_on_write_failure();
}

#[test]
fn cli_rejects_function_rename_without_matching_definition() {
    let dir = fresh_temp_dir("rename-function-missing-definition");
    let lisp_file = dir.join("core.lisp");
    fs::write(&lisp_file, "(old-name 1)\n").expect("write lisp fixture");

    let output = run_rename_function(
        "old-name",
        "new-name",
        None,
        true,
        std::slice::from_ref(&lisp_file),
    );
    assert!(
        !output.status.success(),
        "rename-function unexpectedly succeeded"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("rename-function requires at least one matching callable definition")
    );

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged missing-definition fixture"),
        "(old-name 1)\n"
    );
}

#[test]
fn cli_help_describes_rename_function_contract() {
    let mut cmd = paredit();
    cmd.arg("refactor").arg("rename-function")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Plan or apply a Common Lisp callable definition and callable-designator rename across explicit files",
        ))
        .stdout(predicate::str::contains(
            "including function, macro-function, compiler-macro-function, symbol-function, fdefinition, setf names, and definition forms such as define-method-combination",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--from <FROM>"))
        .stdout(predicate::str::contains("--to <TO>"))
        .stdout(predicate::str::contains("--write"))
        .stdout(predicate::str::contains("--output <OUTPUT>"))
        .stdout(predicate::str::contains("<FILES>..."));
}
