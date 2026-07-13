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
    cmd.arg("refactor")
        .arg("move-definition")
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
fn cli_moves_a_definitions_leading_comment_with_it_and_keeps_neighbors_intact() {
    let dir = fresh_temp_dir("move-definition-comment");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("counting.lisp");
    fs::write(
        &from_file,
        "(defun render-widget (w) w)\n\n\
         ;; Counts the widgets in a list.\n\
         (defun widget-count (widgets) (length widgets))\n\n\
         ;; Trailing helper, unrelated.\n\
         (defun widget-noop () nil)\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("move-definition")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("1")
        .arg("--write")
        .assert()
        .success();

    let source = fs::read_to_string(&from_file).expect("read rewritten source");
    let destination = fs::read_to_string(&to_file).expect("read rewritten destination");
    assert!(
        !source.contains(";; Counts the widgets in a list."),
        "comment must move with its definition, got: {source:?}"
    );
    assert!(
        destination.contains(";; Counts the widgets in a list.\n(defun widget-count"),
        "moved comment must stay directly above its definition, got: {destination:?}"
    );
    assert!(
        !source.contains(")(defun"),
        "remaining definitions must not be glued together, got: {source:?}"
    );
    assert!(source.contains(";; Trailing helper, unrelated.\n(defun widget-noop"));

    let mut check_cmd = paredit();
    check_cmd
        .arg("inspect")
        .arg("check")
        .arg("--file")
        .arg(&from_file)
        .assert()
        .success();
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
    cmd.arg("refactor")
        .arg("move-definition")
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
fn cli_writes_definition_move_into_new_file_with_in_package_header() {
    let dir = fresh_temp_dir("move-definition-new-file-package");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("render.lisp");
    fs::write(
        &from_file,
        "(in-package #:demo)\n(defun keep () :ok)\n(defun render-pane () :render)\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("move-definition")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("2")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"to_file_existed\": false"))
        .stdout(predicate::str::contains("\"written\": true"));

    let destination = fs::read_to_string(&to_file).expect("read new destination file");
    assert!(
        destination.contains("(in-package #:demo)"),
        "new destination file must declare the source package, got: {destination}"
    );
    assert!(destination.contains("(defun render-pane () :render)"));
    let package_index = destination
        .find("(in-package #:demo)")
        .expect("in-package present");
    let definition_index = destination
        .find("(defun render-pane () :render)")
        .expect("definition present");
    assert!(
        package_index < definition_index,
        "in-package must precede the moved definition"
    );
}

#[test]
fn cli_writes_definition_move_between_files_with_matching_package_without_duplicating_in_package() {
    let dir = fresh_temp_dir("move-definition-matching-package");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("render.lisp");
    fs::write(
        &from_file,
        "(in-package #:demo)\n(defun keep () :ok)\n(defun render-pane () :render)\n",
    )
    .expect("write source fixture");
    fs::write(&to_file, "(in-package #:demo)\n(defun boot () :boot)\n")
        .expect("write destination fixture with the same package");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("move-definition")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("2")
        .arg("--write")
        .assert()
        .success();

    let destination = fs::read_to_string(&to_file).expect("read rewritten destination");
    assert_eq!(
        destination.matches("in-package").count(),
        1,
        "must not duplicate an already-matching in-package header, got: {destination}"
    );
}

#[test]
fn cli_writes_symbol_macro_definition_move_between_files() {
    let dir = fresh_temp_dir("move-definition-symbol-macro-write");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("session.lisp");
    fs::write(
        &from_file,
        "(in-package #:demo)\n\
         (define-symbol-macro current-user (slot-value *session* 'user))\n\
         (defun keep () :ok)\n",
    )
    .expect("write source fixture");
    fs::write(&to_file, "(in-package #:demo.session)\n").expect("write destination fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("move-definition")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("1")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"category\": \"variable\""))
        .stdout(predicate::str::contains("\"written\": true"));

    let source = fs::read_to_string(&from_file).expect("read rewritten source");
    let destination = fs::read_to_string(&to_file).expect("read rewritten destination");
    assert!(source.contains("(defun keep () :ok)"));
    assert!(!source.contains("define-symbol-macro current-user"));
    assert!(destination.contains("(in-package #:demo.session)"));
    assert!(
        destination.contains("(define-symbol-macro current-user (slot-value *session* 'user))")
    );
}

#[cfg(unix)]
#[test]
fn cli_rolls_back_definition_move_when_later_file_write_fails() {
    use std::os::unix::fs::PermissionsExt;

    let dir = fresh_temp_dir("move-definition-rollback");
    let from_file = dir.join("core.lisp");
    let nested_dir = dir.join("nested");
    let to_file = nested_dir.join("render.lisp");
    let original_from =
        "(in-package #:demo)\n(defun keep () :ok)\n(defun render-pane () :render)\n";
    fs::write(&from_file, original_from).expect("write source fixture");
    fs::create_dir_all(&nested_dir).expect("create nested dir");
    let original_permissions = fs::metadata(&nested_dir)
        .expect("read nested dir metadata")
        .permissions();
    fs::set_permissions(&nested_dir, PermissionsExt::from_mode(0o555))
        .expect("make nested dir read only");

    let assert_result = paredit()
        .arg("refactor")
        .arg("move-definition")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("2")
        .arg("--write")
        .assert();

    fs::set_permissions(&nested_dir, original_permissions).expect("restore nested dir permissions");

    assert_result
        .failure()
        .stderr(predicate::str::contains("Permission denied"));
    assert_eq!(
        fs::read_to_string(&from_file).expect("read rolled back source"),
        original_from
    );
    assert!(
        !to_file.exists(),
        "destination should not exist after rollback"
    );
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
    cmd.arg("refactor")
        .arg("move-form")
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
fn cli_inserts_one_top_level_form_before_an_anchor() {
    let dir = fresh_temp_dir("insert-top-level-before");
    let file = dir.join("source.lisp");
    fs::write(&file, "(defun existing () :existing)\n").unwrap();

    paredit()
        .args([
            "refactor",
            "insert-top-level",
            "--file",
            file.to_str().unwrap(),
            "--with",
            "(defmacro generated (name) `(list ,name))",
            "--insert",
            "before",
            "--anchor-path",
            "0",
            "--write",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\":true"));

    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "(defmacro generated (name) `(list ,name))\n\n(defun existing () :existing)\n"
    );
}

#[test]
fn cli_rejects_multiple_top_level_forms_for_insertion() {
    let dir = fresh_temp_dir("insert-top-level-multiple");
    let file = dir.join("source.lisp");
    fs::write(&file, "(defun existing () :existing)\n").unwrap();

    paredit()
        .args([
            "refactor",
            "insert-top-level",
            "--file",
            file.to_str().unwrap(),
            "--with",
            "(defun one () 1) (defun two () 2)",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "exactly one top-level S-expression",
        ));
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
    cmd.arg("refactor")
        .arg("move-form")
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
