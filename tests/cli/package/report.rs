use super::*;

#[test]
fn cli_reports_common_lisp_package_declarations() {
    let dir = fresh_temp_dir("package-report");
    let system_file = dir.join("demo.asd");
    let package_file = dir.join("package.lisp");
    fs::write(&system_file, "(asdf:defsystem #:demo)\n").expect("write system fixture");
    fs::write(
        &package_file,
        "(defpackage #:demo\n  (:nicknames #:d)\n  (:use #:cl #:alexandria)\n  (:import-from #:uiop #:pathname-parent-directory-pathname)\n  (:export #:main))\n(in-package #:demo)\n",
    )
    .expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("package-report")
        .arg(&system_file)
        .arg(&package_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"defpackage_count\": 1"))
        .stdout(predicate::str::contains("\"in_package_count\": 1"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"name\": \"#:demo\""))
        .stdout(predicate::str::contains("\"nicknames\""))
        .stdout(predicate::str::contains("\"#:d\""))
        .stdout(predicate::str::contains("\"uses\""))
        .stdout(predicate::str::contains("\"#:alexandria\""))
        .stdout(predicate::str::contains("\"imports\""))
        .stdout(predicate::str::contains("\"package\": \"#:uiop\""))
        .stdout(predicate::str::contains(
            "\"#:pathname-parent-directory-pathname\"",
        ))
        .stdout(predicate::str::contains("\"exports\""))
        .stdout(predicate::str::contains("\"#:main\""));
}
