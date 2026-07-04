use super::*;

#[test]
fn cli_plans_definition_move_between_files_without_writing() {
    let dir = fresh_temp_dir("move-definition-plan");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("render.lisp");
    let original_from = "(in-package #:demo)\n\
                         (defun keep () :ok)\n\
                         (defun render-pane () :render)\n";
    fs::write(&from_file, original_from).expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("move-definition")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("2")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"to_file_existed\": false"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"category\": \"function\""))
        .stdout(predicate::str::contains("\"name\": \"render-pane\""))
        .stdout(predicate::str::contains("(defun render-pane () :render)"));

    assert_eq!(
        fs::read_to_string(&from_file).expect("read unchanged source"),
        original_from
    );
    assert!(
        !to_file.exists(),
        "planning should not create the destination file"
    );
}

#[test]
fn cli_writes_definition_move_between_files() {
    let dir = fresh_temp_dir("move-definition-write");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("render.lisp");
    fs::write(
        &from_file,
        "(in-package #:demo)\n(defun keep () :ok)\n(defun render-pane () :render)\n",
    )
    .expect("write source fixture");
    fs::write(&to_file, "(in-package #:demo.render)\n").expect("write destination fixture");

    let mut cmd = paredit();
    cmd.arg("move-definition")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("2")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"to_file_existed\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    let source = fs::read_to_string(&from_file).expect("read rewritten source");
    let destination = fs::read_to_string(&to_file).expect("read rewritten destination");
    assert!(source.contains("(defun keep () :ok)"));
    assert!(!source.contains("render-pane"));
    assert!(destination.contains("(in-package #:demo.render)"));
    assert!(destination.contains("(defun render-pane () :render)"));
}

#[test]
fn cli_plans_top_level_form_move_without_writing() {
    let dir = fresh_temp_dir("move-form-plan");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("system.lisp");
    let original_from = "(defpackage #:demo (:use #:cl))\n\
                         (in-package #:demo)\n\
                         (eval-when (:compile-toplevel :load-toplevel :execute)\n\
                           (declaim (optimize speed)))\n";
    fs::write(&from_file, original_from).expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("move-form")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("2")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"to_file_existed\": false"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"head\": \"eval-when\""))
        .stdout(predicate::str::contains("\"insert\": \"append\""))
        .stdout(predicate::str::contains("(eval-when"));

    assert_eq!(
        fs::read_to_string(&from_file).expect("read unchanged source"),
        original_from
    );
    assert!(
        !to_file.exists(),
        "planning should not create the destination file"
    );
}

#[test]
fn cli_writes_top_level_form_move_before_anchor() {
    let dir = fresh_temp_dir("move-form-write");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("system.lisp");
    fs::write(
        &from_file,
        "(in-package #:demo)\n(defparameter *feature* t)\n(defun keep () :ok)\n",
    )
    .expect("write source fixture");
    fs::write(
        &to_file,
        "(in-package #:demo.system)\n(defun boot () :boot)\n",
    )
    .expect("write destination fixture");

    let mut cmd = paredit();
    cmd.arg("move-form")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("1")
        .arg("--insert")
        .arg("before")
        .arg("--anchor-path")
        .arg("1")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"to_file_existed\": true"))
        .stdout(predicate::str::contains("\"insert\": \"before\""))
        .stdout(predicate::str::contains("\"anchor_path\": \"1\""))
        .stdout(predicate::str::contains("\"written\": true"));

    let source = fs::read_to_string(&from_file).expect("read rewritten source");
    let destination = fs::read_to_string(&to_file).expect("read rewritten destination");
    assert!(source.contains("(defun keep () :ok)"));
    assert!(!source.contains("*feature*"));
    let moved_index = destination
        .find("(defparameter *feature* t)")
        .expect("moved form should exist");
    let anchor_index = destination
        .find("(defun boot () :boot)")
        .expect("anchor form should exist");
    assert!(moved_index < anchor_index);
}
