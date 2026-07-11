use super::*;

#[test]
fn cli_plans_package_export_sort_without_writing() {
    let dir = fresh_temp_dir("sort-package-exports-plan");
    let package_file = dir.join("package.lisp");
    let original = "(defpackage #:demo\n  (:use #:cl)\n  (:export #:z #:a #:m))\n";
    fs::write(&package_file, original).expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("sort-package-exports")
        .arg("--file")
        .arg(&package_file)
        .arg("--package")
        .arg("demo")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"export_count\": 1"))
        .stdout(predicate::str::contains("\"changed_export_count\": 1"))
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"old_symbols\""))
        .stdout(predicate::str::contains("\"#:z\""))
        .stdout(predicate::str::contains("(:export #:a #:m #:z)"));

    assert_eq!(
        fs::read_to_string(package_file).expect("read unchanged package"),
        original
    );
}

#[test]
fn cli_writes_package_export_sort() {
    let dir = fresh_temp_dir("sort-package-exports-write");
    let package_file = dir.join("package.lisp");
    fs::write(
        &package_file,
        "(defpackage #:demo\n  (:export #:beta #:alpha #:gamma)\n  (:use #:cl))\n",
    )
    .expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("sort-package-exports")
        .arg("--file")
        .arg(&package_file)
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(package_file).expect("read rewritten package"),
        "(defpackage #:demo\n  (:export #:alpha #:beta #:gamma)\n  (:use #:cl))\n"
    );
}

#[test]
fn cli_keeps_sorted_package_exports_idempotent() {
    let dir = fresh_temp_dir("sort-package-exports-idempotent");
    let package_file = dir.join("package.lisp");
    let original = "(defpackage #:demo (:export #:alpha #:beta #:gamma))\n";
    fs::write(&package_file, original).expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("sort-package-exports")
        .arg("--file")
        .arg(&package_file)
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"changed\": false"))
        .stdout(predicate::str::contains("\"written\": false"));

    assert_eq!(
        fs::read_to_string(package_file).expect("read unchanged package"),
        original
    );
}
