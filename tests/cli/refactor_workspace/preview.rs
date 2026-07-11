use super::*;

#[test]
fn cli_previews_workspace_refactor_from_directory_roots() {
    let dir = fresh_temp_dir("workspace refactor preview");
    let file = dir.join("src/core.lisp");
    let ignored = dir.join("src/notes.txt");
    write_fixture(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    );
    write_fixture(&ignored, "render-pane is mentioned in plain text");

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-preview"])
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
        .stdout(predicate::str::contains("\"write_plan\""))
        .stdout(predicate::str::contains("\"write_allowed\": false"))
        .stdout(predicate::str::contains("\"writable_file_count\": 0"))
        .stdout(predicate::str::contains("\"writable_files\": []"))
        .stdout(predicate::str::contains("\"refused_file_count\": 0"))
        .stdout(predicate::str::contains("\"refused_files\": []"))
        .stdout(predicate::str::contains("\"refusal\": null"))
        .stdout(predicate::str::contains("\"decision\""))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains("\"write_parse_refused\": false"))
        .stdout(predicate::str::contains("\"apply_preview\": false"))
        .stdout(predicate::str::contains("\"name\": \"apply-preview\""))
        .stdout(predicate::str::contains("\"status\": \"scheduled\""))
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 1"))
        .stdout(predicate::str::contains("\"unknown\": 1"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"edit_count\": 2"))
        .stdout(predicate::str::contains("\"output_parse_ok\": true"))
        .stdout(predicate::str::contains("\"violation_count\": 0"))
        .stdout(predicate::str::contains("\"write_blocked\": false"))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-preview-or-rerun-with-write\"",
        ))
        .stdout(predicate::str::contains("paint-pane"));
}

#[test]
fn cli_writes_workspace_refactor_preview_after_policy_and_parse_gates() {
    let dir = fresh_temp_dir("workspace refactor preview-write");
    let file = dir.join("src/core.lisp");
    let original = "(defun render-pane (pane) (draw-pane pane))\n(defun caller () (render-pane window) render-pane)\n";
    write_fixture(&file, original);

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-preview"])
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
        .arg("--write")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"write_requested\": true"))
        .stdout(predicate::str::contains("\"write_allowed\": true"))
        .stdout(predicate::str::contains("\"writable_file_count\": 1"))
        .stdout(predicate::str::contains("\"writable_files\": ["))
        .stdout(predicate::str::contains("\"refused_file_count\": 0"))
        .stdout(predicate::str::contains("\"refused_files\": []"))
        .stdout(predicate::str::contains("\"refusal\": null"))
        .stdout(predicate::str::contains("\"decision\""))
        .stdout(predicate::str::contains("\"status\": \"write-applied\""))
        .stdout(predicate::str::contains(
            "\"reason\": \"preview-write-applied\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"run-verification-or-review-diff\"",
        ))
        .stdout(predicate::str::contains("\"write_parse_refused\": false"))
        .stdout(predicate::str::contains("\"apply_preview\": true"))
        .stdout(predicate::str::contains("\"name\": \"apply-preview\""))
        .stdout(predicate::str::contains("\"status\": \"passed\""))
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("\"written_file_count\": 1"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"require_definitions\": 1"))
        .stdout(predicate::str::contains("\"edit_count\": 2"))
        .stdout(predicate::str::contains("\"written\": true"))
        .stdout(predicate::str::contains("\"output_parse_ok\": true"))
        .stdout(predicate::str::contains("\"passed\": true"));

    assert_eq!(
        fs::read_to_string(&file).expect("read rewritten fixture"),
        "(defun paint-pane (pane) (draw-pane pane))\n(defun caller () (paint-pane window) render-pane)\n"
    );
}

#[test]
fn cli_fails_workspace_refactor_preview_definition_policy_after_printing_json() {
    let dir = fresh_temp_dir("workspace refactor preview-definition-policy");
    let file = dir.join("src/core.lisp");
    let original = "(defun render-pane (pane) pane)\n(defun render-pane (pane) (draw-pane pane))\n";
    write_fixture(&file, original);

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--require-definitions")
        .arg("1")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"definition_count\": 2"))
        .stdout(predicate::str::contains("\"require_definitions\": 1"))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "--require-definitions expected exactly 1, found 2",
        ))
        .stderr(predicate::str::contains(
            "refactor workspace-preview policy failed",
        ));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_refuses_workspace_refactor_preview_write_when_policy_fails() {
    let dir = fresh_temp_dir("workspace refactor preview-write-policy");
    let file = dir.join("src/core.lisp");
    let original = "(defun render-pane (pane) pane)\n";
    write_fixture(&file, original);

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--require-edits")
        .arg("3")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"write_requested\": true"))
        .stdout(predicate::str::contains("\"write_allowed\": true"))
        .stdout(predicate::str::contains("\"writable_file_count\": 1"))
        .stdout(predicate::str::contains("\"writable_files\": ["))
        .stdout(predicate::str::contains("\"refused_file_count\": 0"))
        .stdout(predicate::str::contains("\"refused_files\": []"))
        .stdout(predicate::str::contains("\"refusal\": null"))
        .stdout(predicate::str::contains(
            "\"status\": \"blocked-by-policy\"",
        ))
        .stdout(predicate::str::contains(
            "\"reason\": \"preview-policy-failed\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-policy-violations\"",
        ))
        .stdout(predicate::str::contains("\"apply_preview\": false"))
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains("\"violation_count\": 1"))
        .stdout(predicate::str::contains("\"write_blocked\": true"))
        .stdout(predicate::str::contains(
            "--require-edits expected at least 3, found 1",
        ))
        .stderr(predicate::str::contains(
            "refactor workspace-preview policy failed",
        ));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_fails_workspace_refactor_preview_when_target_symbol_already_exists() {
    let dir = fresh_temp_dir("workspace refactor preview-target-conflict");
    let file = dir.join("src/core.lisp");
    let original = "(defun old-name (x) (new-name x))\n";
    write_fixture(&file, original);

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-preview"])
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
        .stdout(predicate::str::contains(
            "\"status\": \"blocked-by-policy\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-policy-violations\"",
        ))
        .stdout(predicate::str::contains("\"apply_preview\": false"))
        .stdout(predicate::str::contains("\"target_occurrence_count\": 1"))
        .stdout(predicate::str::contains(
            "\"fail_on_target_conflict\": true",
        ))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("\"violation_count\": 1"))
        .stdout(predicate::str::contains("\"write_blocked\": true"))
        .stdout(predicate::str::contains(
            "--fail-on-target-conflict found 1 existing replacement symbol occurrence(s)",
        ))
        .stderr(predicate::str::contains(
            "refactor workspace-preview policy failed",
        ));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_previews_workspace_refactor_with_max_depth_limit() {
    let dir = fresh_temp_dir("workspace refactor preview-max-depth");
    let root_file = dir.join("root.lisp");
    let nested_file = dir.join("src/core.lisp");
    write_fixture(
        &root_file,
        "(defun render-pane (pane) pane)\n(defun caller () (render-pane window))\n",
    );
    write_fixture(
        &nested_file,
        "(defun render-pane (pane) (draw-pane pane))\n(defun nested-caller () (render-pane panel))\n",
    );

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--max-depth")
        .arg("1")
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
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 1"))
        .stdout(predicate::str::contains("\"file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"edit_count\": 2"))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains("\"violation_count\": 0"))
        .stdout(predicate::str::contains("\"write_blocked\": false"))
        .stdout(predicate::str::contains("root.lisp"))
        .stdout(predicate::str::contains("core.lisp").not());
}

#[test]
fn cli_previews_workspace_refactor_with_hidden_and_generated_inputs() {
    let dir = fresh_temp_dir("workspace refactor preview-discovery-flags");
    let main_file = dir.join("src/core.lisp");
    let hidden_file = dir.join(".hidden/secret.lisp");
    let generated_file = dir.join("target/generated.lisp");
    write_fixture(
        &main_file,
        "(defun render-pane (pane) (draw-pane pane))\n(defun caller () (render-pane window))\n",
    );
    write_fixture(
        &hidden_file,
        "(defun render-pane (pane) (draw-pane pane))\n(defun hidden-caller () (render-pane hidden-window))\n",
    );
    write_fixture(
        &generated_file,
        "(defun render-pane (pane) (draw-pane pane))\n(defun generated-caller () (render-pane generated-window))\n",
    );

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--include-hidden")
        .arg("--include-generated")
        .arg("--require-changed-files")
        .arg("3")
        .arg("--require-definitions")
        .arg("3")
        .arg("--require-edits")
        .arg("6")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 3"))
        .stdout(predicate::str::contains("\"hidden\": 0"))
        .stdout(predicate::str::contains("\"generated\": 0"))
        .stdout(predicate::str::contains("\"unknown\": 0"))
        .stdout(predicate::str::contains("\"file_count\": 3"))
        .stdout(predicate::str::contains("\"changed_file_count\": 3"))
        .stdout(predicate::str::contains("\"definition_count\": 3"))
        .stdout(predicate::str::contains("\"edit_count\": 6"))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains("\"violation_count\": 0"))
        .stdout(predicate::str::contains("\"write_blocked\": false"))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("secret.lisp"))
        .stdout(predicate::str::contains("generated.lisp"));
}

#[test]
fn cli_previews_workspace_refactor_with_unknown_inputs() {
    let dir = fresh_temp_dir("workspace refactor preview-unknown");
    let main_file = dir.join("src/core.lisp");
    let unknown_file = dir.join("src/scratch.txt");
    write_fixture(
        &main_file,
        "(defun render-pane (pane) pane)\n(defun caller () (render-pane window))\n",
    );
    write_fixture(
        &unknown_file,
        "(defun unknown-caller () (render-pane preview-window))\n",
    );

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--include-unknown")
        .arg("--fail-on-no-change")
        .arg("--require-changed-files")
        .arg("2")
        .arg("--require-definitions")
        .arg("1")
        .arg("--require-edits")
        .arg("3")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 2"))
        .stdout(predicate::str::contains("\"unknown\": 0"))
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"edit_count\": 3"))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains("\"violation_count\": 0"))
        .stdout(predicate::str::contains("\"write_blocked\": false"))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("scratch.txt"));
}
