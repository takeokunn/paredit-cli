use super::*;

#[test]
fn readme_quickstart_commands_smoke() {
    let dir = fresh_temp_dir("readme-smoke");
    let src_dir = dir.join("src");
    let lisp_dir = dir.join("lisp");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::create_dir_all(&lisp_dir).expect("create lisp dir");

    let source_file = src_dir.join("source.lisp");
    let helper_file = lisp_dir.join("helpers.el");
    fs::write(
        &source_file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun old-name (x) (list x))
(defun caller () (old-name 1))
"#,
    )
    .expect("write source fixture");
    fs::write(
        &helper_file,
        r#"(defun helper-entry ()
  (old-name 2))
"#,
    )
    .expect("write helper fixture");

    let source_file_arg = source_file.display().to_string();
    let workspace_arg = dir.display().to_string();
    let helper_file_arg = helper_file.display().to_string();

    paredit()
        .args(["inspect", "check", "--file", &source_file_arg])
        .assert()
        .success();

    paredit()
        .args(["inspect", "workspace", "--output", "json", &workspace_arg])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"parsed_count\": 2"))
        .stdout(predicate::str::contains("\"definition_count\": 3"));

    paredit()
        .args([
            "inspect",
            "form",
            "--file",
            &source_file_arg,
            "--path",
            "0",
            "--include-source",
            "--output",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"head\": \"defpackage\""))
        .stdout(predicate::str::contains(
            "\"source\": \"(defpackage #:demo.core (:use #:cl))\"",
        ));

    paredit()
        .args([
            "refactor",
            "plan",
            "--symbol",
            "old-name",
            "--operation",
            "rename",
            "--fail-on-blocking-gate",
            "--require-definitions",
            "1",
            "--require-references",
            "1",
            "--output",
            "json",
            &source_file_arg,
            &helper_file_arg,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ready\""))
        .stdout(predicate::str::contains("\"symbol\": \"old-name\""))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"reference_count\": 2"));

    paredit()
        .args([
            "refactor",
            "preview",
            "--from",
            "old-name",
            "--to",
            "new-name",
            "--mode",
            "function",
            "--fail-on-no-change",
            "--fail-on-parse-error",
            "--fail-on-target-conflict",
            "--require-definitions",
            "1",
            "--require-edits",
            "3",
            "--output",
            "json",
            &source_file_arg,
            &helper_file_arg,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .stdout(predicate::str::contains("\"edit_count\": 3"))
        .stdout(predicate::str::contains("\"replacement\": \"new-name\""));

    paredit()
        .args([
            "refactor",
            "verify",
            "--symbol",
            "old-name",
            "--new-symbol",
            "new-name",
            "--operation",
            "rename",
            "--phase",
            "post",
            "--output",
            "json",
            &source_file_arg,
            &helper_file_arg,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"phase\": \"post\""))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""));
}
