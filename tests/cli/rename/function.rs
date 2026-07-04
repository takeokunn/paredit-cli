use super::*;

#[test]
fn cli_plans_function_rename_without_renaming_value_references() {
    let dir = fresh_temp_dir("rename-function-plan");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"path\": \"0.1\""))
        .stdout(predicate::str::contains("\"path\": \"1.3.0\""));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_function_rename_across_files() {
    let dir = fresh_temp_dir("rename-function-write");
    let definition_file = dir.join("core.lisp");
    let caller_file = dir.join("caller.el");
    fs::write(&definition_file, "(defun old-name (x) x)\n").expect("write definition fixture");
    fs::write(&caller_file, "(defun caller () (old-name 1) old-name)\n")
        .expect("write caller fixture");

    let mut cmd = paredit();
    cmd.arg("rename-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&definition_file)
        .arg(&caller_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&definition_file).expect("read rewritten definition fixture"),
        "(defun new-name (x) x)\n"
    );
    assert_eq!(
        fs::read_to_string(&caller_file).expect("read rewritten caller fixture"),
        "(defun caller () (new-name 1) old-name)\n"
    );
}

#[test]
fn cli_rejects_function_rename_without_matching_definition() {
    let dir = fresh_temp_dir("rename-function-missing-definition");
    let lisp_file = dir.join("core.lisp");
    fs::write(&lisp_file, "(old-name 1)\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rename-function requires at least one matching callable definition",
        ));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged missing-definition fixture"),
        "(old-name 1)\n"
    );
}
