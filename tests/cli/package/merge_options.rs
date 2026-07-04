use super::*;

#[test]
fn cli_plans_package_option_merge_without_writing() {
    let dir = fresh_temp_dir("merge-package-options-plan");
    let package_file = dir.join("package.lisp");
    let original = "(defpackage #:demo\n  (:use #:cl)\n  (:export #:b #:a)\n  (:export #:a #:c)\n  (:import-from #:dep #:x)\n  (:import-from #:dep #:x #:y))\n";
    fs::write(&package_file, original).expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("merge-package-options")
        .arg("--file")
        .arg(&package_file)
        .arg("--package")
        .arg("demo")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"merge_count\": 2"))
        .stdout(predicate::str::contains("\"changed_merge_count\": 2"))
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"head\": \":export\""))
        .stdout(predicate::str::contains("\"key\": \"dep\""))
        .stdout(predicate::str::contains("(:export #:b #:a #:c)"))
        .stdout(predicate::str::contains("(:import-from #:dep #:x #:y)"));

    assert_eq!(
        fs::read_to_string(package_file).expect("read unchanged package"),
        original
    );
}

#[test]
fn cli_writes_package_option_merge() {
    let dir = fresh_temp_dir("merge-package-options-write");
    let package_file = dir.join("package.lisp");
    fs::write(
        &package_file,
        "(defpackage #:demo\n  (:export #:b)\n  (:export #:a)\n  (:use #:cl))\n",
    )
    .expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("merge-package-options")
        .arg("--file")
        .arg(&package_file)
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    let rewritten = fs::read_to_string(&package_file).expect("read rewritten package");
    assert!(rewritten.contains("(:export #:b #:a)"));
    assert!(!rewritten.contains("(:export #:a)"));

    let mut check = paredit();
    check
        .arg("check")
        .arg("--file")
        .arg(&package_file)
        .assert()
        .success();
}

#[test]
fn cli_keeps_merged_package_options_idempotent() {
    let dir = fresh_temp_dir("merge-package-options-idempotent");
    let package_file = dir.join("package.lisp");
    let original = "(defpackage #:demo (:use #:cl) (:export #:main))\n";
    fs::write(&package_file, original).expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("merge-package-options")
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
