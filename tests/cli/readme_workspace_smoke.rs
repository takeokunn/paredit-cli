use super::*;

#[test]
fn readme_workspace_refactor_commands_smoke() {
    let dir = fresh_temp_dir("readme-workspace-smoke");
    let src_dir = dir.join("src");
    let lisp_dir = dir.join("lisp");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::create_dir_all(&lisp_dir).expect("create lisp dir");

    let source_file = src_dir.join("core.lisp");
    let helper_file = lisp_dir.join("helpers.el");
    fs::write(
        &source_file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window))
"#,
    )
    .expect("write source fixture");
    fs::write(
        &helper_file,
        r#"(defun helper-entry ()
  (render-pane sidebar))
"#,
    )
    .expect("write helper fixture");

    let workspace_arg = dir.display().to_string();
    let source_file_arg = source_file.display().to_string();
    let helper_file_arg = helper_file.display().to_string();

    paredit()
        .args([
            "refactor",
            "workspace-plan",
            "--symbol",
            "render-pane",
            "--operation",
            "rename",
            "--output",
            "json",
            &workspace_arg,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"rename\""))
        .stdout(predicate::str::contains("\"symbol\": \"render-pane\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 2"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 2"));

    paredit()
        .args([
            "refactor",
            "workspace-preview",
            "--from",
            "render-pane",
            "--to",
            "paint-pane",
            "--mode",
            "function",
            "--fail-on-no-change",
            "--fail-on-parse-error",
            "--require-definitions",
            "1",
            "--require-edits",
            "3",
            "--output",
            "json",
            &workspace_arg,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .stdout(predicate::str::contains("\"edit_count\": 3"))
        .stdout(predicate::str::contains("\"replacement\": \"paint-pane\""));

    paredit()
        .args([
            "refactor",
            "workspace-execute",
            "--from",
            "render-pane",
            "--to",
            "paint-pane",
            "--mode",
            "function",
            "--write",
            "--fail-on-no-change",
            "--fail-on-parse-error",
            "--require-changed-files",
            "2",
            "--require-definitions",
            "1",
            "--require-edits",
            "3",
            "--output",
            "json",
            &workspace_arg,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"command\": \"refactor workspace-execute\"",
        ))
        .stdout(predicate::str::contains("\"status\": \"write-applied\""))
        .stdout(predicate::str::contains("\"written_file_count\": 2"))
        .stdout(predicate::str::contains(
            "\"post_verification_passed\": true",
        ));

    paredit()
        .args([
            "refactor",
            "verify",
            "--symbol",
            "render-pane",
            "--new-symbol",
            "paint-pane",
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
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""));
}
