use super::*;

#[test]
fn cli_plans_package_rename_without_writing() {
    let dir = fresh_temp_dir("rename-package-plan");
    let file = dir.join("package.lisp");
    let original = "(defpackage #:old.pkg\n\
                      (:use #:cl #:old.pkg)\n\
                      (:import-from #:old.pkg #:thing))\n\
                    (in-package #:old.pkg)\n\
                    (defun call () (old.pkg:thing old.pkg::internal same-name))\n\
                    \"old.pkg:string\"\n\
                    ;; old.pkg:comment\n";
    fs::write(&file, original).expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("rename-package")
        .arg("--from")
        .arg("old.pkg")
        .arg("--to")
        .arg("new.pkg")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"occurrence_count\": 6"))
        .stdout(predicate::str::contains("\"kind\": \"defpackage-name\""))
        .stdout(predicate::str::contains("\"kind\": \"in-package-name\""))
        .stdout(predicate::str::contains("\"kind\": \"package-option\""))
        .stdout(predicate::str::contains("\"kind\": \"qualified-prefix\""))
        .stdout(predicate::str::contains("\"replacement\": \"#:new.pkg\""))
        .stdout(predicate::str::contains(
            "\"replacement\": \"new.pkg:thing\"",
        ))
        .stdout(predicate::str::contains(
            "\"replacement\": \"new.pkg::internal\"",
        ));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged package fixture"),
        original
    );
}

#[test]
fn cli_writes_package_rename() {
    let dir = fresh_temp_dir("rename-package-write");
    let file = dir.join("package.lisp");
    fs::write(
        &file,
        "(defpackage :old.pkg (:use :cl))\n\
         (in-package :old.pkg)\n\
         (defun call () old.pkg:thing)\n\
         \"old.pkg:literal\"\n",
    )
    .expect("write package fixture");

    let mut cmd = paredit();
    cmd.arg("rename-package")
        .arg("--from")
        .arg(":old.pkg")
        .arg("--to")
        .arg(":new.pkg")
        .arg("--write")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written_count\": 1"));

    let rewritten = fs::read_to_string(&file).expect("read renamed package fixture");
    assert!(rewritten.contains("(defpackage :new.pkg"));
    assert!(rewritten.contains("(in-package :new.pkg)"));
    assert!(rewritten.contains("new.pkg:thing"));
    assert!(rewritten.contains("\"old.pkg:literal\""));
    assert!(!rewritten.contains("old.pkg:thing"));
}
