use super::*;

#[test]
fn cli_previews_workspace_refactor_from_directory_roots() {
    let dir = fresh_temp_dir("workspace-refactor-preview");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let file = src_dir.join("core.lisp");
    let ignored = src_dir.join("notes.txt");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    )
    .expect("write workspace refactor preview fixture");
    fs::write(&ignored, "render-pane is mentioned in plain text")
        .expect("write ignored workspace refactor preview fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-preview")
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
        .arg("--fail-on-parse-error")
        .arg("--require-changed-files")
        .arg("1")
        .arg("--require-definitions")
        .arg("1")
        .arg("--require-edits")
        .arg("2")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"mode\": \"function\""))
        .stdout(predicate::str::contains("\"from\": \"render-pane\""))
        .stdout(predicate::str::contains("\"to\": \"paint-pane\""))
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 1"))
        .stdout(predicate::str::contains("\"unknown\": 1"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"edit_count\": 2"))
        .stdout(predicate::str::contains("\"output_parse_ok\": true"))
        .stdout(predicate::str::contains("paint-pane"));
}

#[test]
fn cli_fails_workspace_refactor_preview_when_target_symbol_already_exists() {
    let dir = fresh_temp_dir("workspace-refactor-preview-target-conflict");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let file = src_dir.join("core.lisp");
    let original = "(defun old-name (x) (new-name x))\n";
    fs::write(&file, original).expect("write workspace refactor preview fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-preview")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("symbol")
        .arg("--fail-on-target-conflict")
        .arg("--require-changed-files")
        .arg("1")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"target_occurrence_count\": 1"))
        .stdout(predicate::str::contains(
            "\"fail_on_target_conflict\": true",
        ))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains(
            "--fail-on-target-conflict found 1 existing replacement symbol occurrence(s)",
        ))
        .stderr(predicate::str::contains("refactor-preview policy failed"));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged fixture"),
        original
    );
}
