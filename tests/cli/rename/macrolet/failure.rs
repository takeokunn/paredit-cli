use super::*;

#[test]
fn cli_rejects_macrolet_rename_without_matching_definition() {
    let dir = fresh_temp_dir("rename-macrolet-missing-definition");
    let lisp_file = dir.join("core.lisp");
    write_fixture(&lisp_file, "(old-name 1)\n", "missing-definition fixture");

    let mut cmd = paredit();
    cmd.arg("refactor").arg("rename-macrolet")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rename-macrolet requires at least one matching macrolet or compiler-macrolet definition",
        ));

    assert_eq!(
        read_fixture(&lisp_file, "unchanged missing-definition fixture"),
        "(old-name 1)\n"
    );
}

#[test]
fn cli_help_describes_rename_macrolet_contract() {
    let mut cmd = paredit();
    cmd.arg("refactor").arg("rename-macrolet")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Plan or apply a Common Lisp macrolet/compiler-macrolet binding and call-site rename across explicit files",
        ))
        .stdout(predicate::str::contains(
            "while keeping expander bodies out of scope",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--from <FROM>"))
        .stdout(predicate::str::contains("--to <TO>"))
        .stdout(predicate::str::contains("--write"))
        .stdout(predicate::str::contains("--output <OUTPUT>"))
        .stdout(predicate::str::contains("<FILES>..."));
}
