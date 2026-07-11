use super::*;

#[test]
fn cli_plans_definition_removal_without_writing() {
    let dir = fresh_temp_dir("remove-definition-plan");
    let file = dir.join("core.lisp");
    let original = "(in-package #:demo)\n\
                    (defun keep () :ok)\n\
                    (defun stale-helper () :stale)\n";
    fs::write(&file, original).expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("remove-definition")
        .arg("--file")
        .arg(&file)
        .arg("--path")
        .arg("2")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"category\": \"function\""))
        .stdout(predicate::str::contains("\"name\": \"stale-helper\""))
        .stdout(predicate::str::contains("(defun stale-helper () :stale)"));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_writes_definition_removal() {
    let dir = fresh_temp_dir("remove-definition-write");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(in-package #:demo)\n(defun keep () :ok)\n(defun stale-helper () :stale)\n",
    )
    .expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("remove-definition")
        .arg("--file")
        .arg(&file)
        .arg("--path")
        .arg("2")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    let rewritten = fs::read_to_string(&file).expect("read rewritten fixture");
    assert!(rewritten.contains("(in-package #:demo)"));
    assert!(rewritten.contains("(defun keep () :ok)"));
    assert!(!rewritten.contains("stale-helper"));
}

#[test]
fn cli_plans_unused_definition_removal_without_writing() {
    let dir = fresh_temp_dir("remove-unused-definitions-plan");
    let file = dir.join("core.lisp");
    let original = "(defun used () :ok)\n\
                    (defun caller () (used))\n\
                    (caller)\n\
                    (defun stale-helper () :stale)\n\
                    (deftest stale-test () :test)\n";
    fs::write(&file, original).expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-definitions")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"candidate_count\": 2"))
        .stdout(predicate::str::contains("\"removal_count\": 1"))
        .stdout(predicate::str::contains("\"skipped_count\": 1"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"name\": \"stale-helper\""))
        .stdout(predicate::str::contains("\"name\": \"stale-test\""))
        .stdout(predicate::str::contains("protected-definition-category"));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_writes_unused_definition_removal() {
    let dir = fresh_temp_dir("remove-unused-definitions-write");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(defun used () :ok)\n\
         (defun caller () (used))\n\
         (caller)\n\
         (defun stale-helper-a () :stale)\n\
         (defun stale-helper-b () :stale)\n\
         (deftest stale-test () :test)\n",
    )
    .expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-definitions")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"removal_count\": 2"))
        .stdout(predicate::str::contains("\"skipped_count\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    let rewritten = fs::read_to_string(&file).expect("read rewritten fixture");
    assert!(rewritten.contains("(defun used () :ok)"));
    assert!(rewritten.contains("(defun caller () (used))"));
    assert!(rewritten.contains("(caller)"));
    assert!(rewritten.contains("(deftest stale-test () :test)"));
    assert!(!rewritten.contains("stale-helper-a"));
    assert!(!rewritten.contains("stale-helper-b"));
}

#[test]
fn cli_keeps_exported_unused_definition_by_default() {
    let dir = fresh_temp_dir("remove-unused-definitions-exported-plan");
    let file = dir.join("core.lisp");
    let original = concat!(
        "(defpackage #:demo\n",
        "  (:use #:cl)\n",
        "  (:export #:public-entry))\n",
        "(in-package #:demo)\n",
        "(defun public-entry () :api)\n",
        "(defun stale-private () :private)\n",
    );
    fs::write(&file, original).expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-definitions")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"removal_count\": 1"))
        .stdout(predicate::str::contains("\"skipped_count\": 1"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"name\": \"public-entry\""))
        .stdout(predicate::str::contains("\"name\": \"stale-private\""))
        .stdout(predicate::str::contains("exported-definition"));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_keeps_exported_unused_definition_when_in_package_uses_nickname() {
    let dir = fresh_temp_dir("remove-unused-definitions-exported-nickname-plan");
    let file = dir.join("core.lisp");
    let original = concat!(
        "(defpackage #:demo.core\n",
        "  (:nicknames #:core)\n",
        "  (:use #:cl)\n",
        "  (:export #:public-entry))\n",
        "(in-package #:core)\n",
        "(defun public-entry () :api)\n",
        "(defun stale-private () :private)\n",
    );
    fs::write(&file, original).expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-definitions")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"removal_count\": 1"))
        .stdout(predicate::str::contains("\"skipped_count\": 2"))
        .stdout(predicate::str::contains("\"name\": \"public-entry\""))
        .stdout(predicate::str::contains("exported-definition"));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_removes_exported_unused_definition_when_requested() {
    let dir = fresh_temp_dir("remove-unused-definitions-exported-write");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        concat!(
            "(defpackage #:demo\n",
            "  (:use #:cl)\n",
            "  (:export #:public-entry))\n",
            "(in-package #:demo)\n",
            "(defun public-entry () :api)\n",
            "(defun stale-private () :private)\n",
        ),
    )
    .expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("remove-unused-definitions")
        .arg("--include-exported")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"removal_count\": 2"))
        .stdout(predicate::str::contains("\"skipped_count\": 0"))
        .stdout(predicate::str::contains("\"written\": true"));

    let rewritten = fs::read_to_string(&file).expect("read rewritten fixture");
    assert!(rewritten.contains("(defpackage #:demo"));
    assert!(rewritten.contains("(in-package #:demo)"));
    assert!(!rewritten.contains("public-entry ()"));
    assert!(!rewritten.contains("stale-private"));
}
