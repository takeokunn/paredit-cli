use super::*;

#[test]
fn cli_plans_flet_rename_without_touching_definition_body_or_noncall_values() {
    let dir = fresh_temp_dir("rename-local-function-flet-plan");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains(
            "(flet ((new-name (x) (old-name x))) (new-name 1) old-name)",
        ));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        "(flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_labels_rename_with_recursive_calls() {
    let dir = fresh_temp_dir("rename-local-function-labels-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(labels ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write labels fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 2"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten labels fixture"),
        "(labels ((new-name (x) (new-name x))) (new-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_local_function_rename_without_crossing_nested_shadow() {
    let dir = fresh_temp_dir("rename-local-function-shadow-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(flet ((old-name (x) x)) (labels ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
    )
    .expect("write shadow fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten shadow fixture"),
        "(flet ((new-name (x) x)) (labels ((old-name (y) (old-name y))) (old-name 1)) (new-name 2))\n"
    );
}

#[test]
fn cli_rejects_local_function_rename_without_matching_definition() {
    let dir = fresh_temp_dir("rename-local-function-missing-definition");
    let lisp_file = dir.join("core.lisp");
    fs::write(&lisp_file, "(old-name 1)\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rename-local-function requires at least one matching flet or labels definition",
        ));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged missing-definition fixture"),
        "(old-name 1)\n"
    );
}
