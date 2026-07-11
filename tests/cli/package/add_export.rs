use super::*;

#[test]
fn cli_plans_common_lisp_export_addition() {
    let dir = fresh_temp_dir("add-export-plan");
    let package_file = dir.join("package.lisp");
    let original = "(defpackage #:demo\n  (:use #:cl)\n  (:export #:old))\n";
    fs::write(&package_file, original).expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("add-export")
        .arg("--file")
        .arg(&package_file)
        .arg("--package")
        .arg("demo")
        .arg("--symbol")
        .arg("#:new")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"package\": \"#:demo\""))
        .stdout(predicate::str::contains("\"symbol\": \"#:new\""))
        .stdout(predicate::str::contains("\"already_exported\": false"))
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("(:export #:old #:new)"));

    assert_eq!(
        fs::read_to_string(package_file).expect("read unchanged package"),
        original
    );
}

#[test]
fn cli_writes_common_lisp_export_addition() {
    let dir = fresh_temp_dir("add-export-write");
    let package_file = dir.join("package.lisp");
    fs::write(&package_file, "(defpackage #:demo\n  (:use #:cl))\n")
        .expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("add-export")
        .arg("--file")
        .arg(&package_file)
        .arg("--symbol")
        .arg("#:main")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"already_exported\": false"))
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(package_file).expect("read rewritten package"),
        "(defpackage #:demo\n  (:use #:cl)\n  (:export #:main))\n"
    );
}

#[test]
fn cli_keeps_existing_common_lisp_export_idempotent() {
    let dir = fresh_temp_dir("add-export-idempotent");
    let package_file = dir.join("package.lisp");
    let original = "(defpackage #:demo (:export #:main))\n";
    fs::write(&package_file, original).expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("add-export")
        .arg("--file")
        .arg(&package_file)
        .arg("--package")
        .arg(":demo")
        .arg("--symbol")
        .arg("main")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"already_exported\": true"))
        .stdout(predicate::str::contains("\"changed\": false"))
        .stdout(predicate::str::contains("\"written\": false"));

    assert_eq!(
        fs::read_to_string(package_file).expect("read unchanged package"),
        original
    );
}
