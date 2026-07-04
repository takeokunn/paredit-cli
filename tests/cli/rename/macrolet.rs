use super::*;

#[test]
fn cli_plans_macrolet_rename_without_touching_noncall_values() {
    let dir = fresh_temp_dir("rename-macrolet-plan");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-macrolet")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        "(macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n"
    );
}

#[test]
fn cli_plans_compiler_macrolet_rename_without_touching_noncall_values() {
    let dir = fresh_temp_dir("rename-compiler-macrolet-plan");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(compiler-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-macrolet")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        "(compiler-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_macrolet_rename_across_files() {
    let dir = fresh_temp_dir("rename-macrolet-write");
    let macrolet_file = dir.join("core.lisp");
    fs::write(
        &macrolet_file,
        "(macrolet ((old-name (x) (list old-name x))) (old-name 1))\n",
    )
    .expect("write macrolet fixture");

    let mut cmd = paredit();
    cmd.arg("rename-macrolet")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&macrolet_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&macrolet_file).expect("read rewritten macrolet fixture"),
        "(macrolet ((new-name (x) (list old-name x))) (new-name 1))\n"
    );
}

#[test]
fn cli_writes_macrolet_rename_without_crossing_nested_macrolet_shadow() {
    let dir = fresh_temp_dir("rename-macrolet-nested-shadow-write");
    let macrolet_file = dir.join("core.lisp");
    fs::write(
        &macrolet_file,
        "(macrolet ((old-name (x) x)) (macrolet ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
    )
    .expect("write nested macrolet fixture");

    let mut cmd = paredit();
    cmd.arg("rename-macrolet")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&macrolet_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&macrolet_file).expect("read rewritten nested macrolet fixture"),
        "(macrolet ((new-name (x) x)) (macrolet ((old-name (y) (old-name y))) (old-name 1)) (new-name 2))\n"
    );
}

#[test]
fn cli_rejects_macrolet_rename_without_matching_definition() {
    let dir = fresh_temp_dir("rename-macrolet-missing-definition");
    let lisp_file = dir.join("core.lisp");
    fs::write(&lisp_file, "(old-name 1)\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-macrolet")
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
        fs::read_to_string(&lisp_file).expect("read unchanged missing-definition fixture"),
        "(old-name 1)\n"
    );
}
