use super::*;

#[test]
fn cli_plans_package_option_sort_without_writing() {
    let dir = fresh_temp_dir("sort-package-options-plan");
    let package_file = dir.join("package.lisp");
    let original =
        "(defpackage #:demo\n  (:export #:main)\n  (:use #:cl)\n  (:import-from #:dep #:x))\n";
    fs::write(&package_file, original).expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("sort-package-options")
        .arg("--file")
        .arg(&package_file)
        .arg("--package")
        .arg("demo")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"package_count\": 1"))
        .stdout(predicate::str::contains("\"changed_package_count\": 1"))
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"old_options\""))
        .stdout(predicate::str::contains("\"new_options\""))
        .stdout(predicate::str::contains("(:use #:cl)"))
        .stdout(predicate::str::contains("(:import-from #:dep #:x)"))
        .stdout(predicate::str::contains("(:export #:main)"));

    assert_eq!(
        fs::read_to_string(package_file).expect("read unchanged package"),
        original
    );
}

#[test]
fn cli_writes_package_option_sort() {
    let dir = fresh_temp_dir("sort-package-options-write");
    let package_file = dir.join("package.lisp");
    fs::write(
        &package_file,
        "(defpackage #:demo\n  (:export #:main)\n  (:import-from #:dep #:x)\n  (:use #:cl))\n",
    )
    .expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("sort-package-options")
        .arg("--file")
        .arg(&package_file)
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(package_file).expect("read rewritten package"),
        "(defpackage #:demo\n  (:use #:cl)\n  (:import-from #:dep #:x)\n  (:export #:main))\n"
    );
}

#[test]
fn cli_keeps_sorted_package_options_idempotent() {
    let dir = fresh_temp_dir("sort-package-options-idempotent");
    let package_file = dir.join("package.lisp");
    let original = "(defpackage #:demo (:use #:cl) (:import-from #:dep #:x) (:export #:main))\n";
    fs::write(&package_file, original).expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("sort-package-options")
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
