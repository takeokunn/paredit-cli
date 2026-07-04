use super::*;

#[test]
fn cli_e2e_applies_workspace_refactor_manifest_across_lisp_dialects() {
    let dir = fresh_temp_dir("workspace-refactor-apply");
    let src_dir = dir.join("src");
    let manifest_file = dir.join("workspace.preview.json");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let lisp_file = src_dir.join("core.lisp");
    let elisp_file = src_dir.join("ui.el");
    let ignored = src_dir.join("notes.txt");
    fs::write(
        &lisp_file,
        "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n",
    )
    .expect("write common lisp fixture");
    fs::write(
        &elisp_file,
        "(defun ui () (old-name 2))\n(message \"old-name\")\n",
    )
    .expect("write emacs lisp fixture");
    fs::write(&ignored, "old-name is plain text").expect("write ignored fixture");

    let mut preview = paredit();
    let preview_output = preview
        .arg("workspace-refactor-preview")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
        .arg("--fail-on-parse-error")
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
        .stdout(predicate::str::contains("\"discovered_file_count\": 2"))
        .stdout(predicate::str::contains("\"unknown\": 1"))
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .get_output()
        .stdout
        .clone();
    fs::write(&manifest_file, preview_output).expect("write workspace manifest");

    let mut apply = paredit();
    apply
        .arg("refactor-apply")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"mode\": \"function\""))
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .stdout(predicate::str::contains("\"written_file_count\": 2"))
        .stdout(predicate::str::contains("\"edit_count\": 3"))
        .stdout(predicate::str::contains("\"applied\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten common lisp fixture"),
        "(defun new-name (x) x)\n(defun caller () (new-name 1) old-name)\n"
    );
    assert_eq!(
        fs::read_to_string(&elisp_file).expect("read rewritten emacs lisp fixture"),
        "(defun ui () (new-name 2))\n(message \"old-name\")\n"
    );
    assert_eq!(
        fs::read_to_string(&ignored).expect("read ignored fixture"),
        "old-name is plain text"
    );
}

#[test]
fn cli_builds_gated_refactor_plan_for_agents() {
    let dir = fresh_temp_dir("refactor-plan");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    )
    .expect("write refactor plan fixture");

    let mut cmd = paredit();
    cmd.arg("refactor-plan")
        .arg("--symbol")
        .arg("render-pane")
        .arg("--operation")
        .arg("rename")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"rename\""))
        .stdout(predicate::str::contains("\"symbol\": \"render-pane\""))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains(
            "\"code\": \"non-call-references\"",
        ))
        .stdout(predicate::str::contains(
            "\"action\": \"run-impact-report\"",
        ))
        .stdout(predicate::str::contains("--fail-on-risk-level warning"))
        .stdout(predicate::str::contains("--require-definitions 1"))
        .stdout(predicate::str::contains("--require-references 1"))
        .stdout(predicate::str::contains("--require-calls 1"))
        .stdout(predicate::str::contains(
            "paredit rename-symbols --from 'render-pane' --to <new-symbol> --output json",
        ));
}

#[test]
fn cli_builds_workspace_refactor_plan_from_directory_roots() {
    let dir = fresh_temp_dir("workspace-refactor-plan");
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
    .expect("write workspace refactor fixture");
    fs::write(&ignored, "render-pane is mentioned in plain text")
        .expect("write ignored workspace fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-plan")
        .arg("--symbol")
        .arg("render-pane")
        .arg("--operation")
        .arg("rename")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"rename\""))
        .stdout(predicate::str::contains("\"symbol\": \"render-pane\""))
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 1"))
        .stdout(predicate::str::contains("\"unknown\": 1"))
        .stdout(predicate::str::contains("\"file_count\": 1"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains(
            "paredit rename-symbols --from 'render-pane' --to <new-symbol> --output json",
        ));
}

#[test]
fn cli_builds_workspace_remove_plan_with_unused_definition_cleanup_command() {
    let dir = fresh_temp_dir("workspace-remove-plan");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let file = src_dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun stale-helper (value) value)
(defun caller () 42)
"#,
    )
    .expect("write workspace remove fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-plan")
        .arg("--symbol")
        .arg("stale-helper")
        .arg("--operation")
        .arg("remove")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"remove\""))
        .stdout(predicate::str::contains("\"symbol\": \"stale-helper\""))
        .stdout(predicate::str::contains(
            "\"action\": \"apply-unused-definition-removal\"",
        ))
        .stdout(predicate::str::contains(
            "paredit remove-unused-definitions --output json",
        ))
        .stdout(predicate::str::contains(
            "paredit verify-refactor --symbol 'stale-helper' --operation remove --phase post --output json",
        ));
}

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

#[test]
fn cli_executes_workspace_refactor_with_post_verification() {
    let dir = fresh_temp_dir("workspace-refactor-execute");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let lisp_file = src_dir.join("core.lisp");
    let elisp_file = src_dir.join("ui.el");
    let ignored = src_dir.join("notes.txt");
    fs::write(
        &lisp_file,
        "(defun old-name (x) (list x))\n(defun caller () (old-name 1))\n",
    )
    .expect("write common lisp fixture");
    fs::write(
        &elisp_file,
        "(defun ui () (old-name 2))\n(message \"old-name\")\n",
    )
    .expect("write emacs lisp fixture");
    fs::write(&ignored, "old-name is plain text").expect("write ignored fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-execute")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("symbol")
        .arg("--write")
        .arg("--fail-on-no-change")
        .arg("--fail-on-parse-error")
        .arg("--require-changed-files")
        .arg("2")
        .arg("--require-edits")
        .arg("3")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"command\": \"workspace-refactor-execute\"",
        ))
        .stdout(predicate::str::contains("\"mode\": \"symbol\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 2"))
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .stdout(predicate::str::contains("\"written_file_count\": 2"))
        .stdout(predicate::str::contains("\"pre_verification\""))
        .stdout(predicate::str::contains("\"post_verification\""))
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains("\"code\": \"preflight-gates\""))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten common lisp fixture"),
        "(defun new-name (x) (list x))\n(defun caller () (new-name 1))\n"
    );
    assert_eq!(
        fs::read_to_string(&elisp_file).expect("read rewritten emacs lisp fixture"),
        "(defun ui () (new-name 2))\n(message \"old-name\")\n"
    );
    assert_eq!(
        fs::read_to_string(&ignored).expect("read ignored fixture"),
        "old-name is plain text"
    );
}

#[test]
fn cli_dry_runs_workspace_refactor_execute_without_writing() {
    let dir = fresh_temp_dir("workspace-refactor-execute-dry-run");
    let file = dir.join("core.lisp");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1))\n";
    fs::write(&file, original).expect("write workspace refactor execute fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-execute")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("symbol")
        .arg("--fail-on-no-change")
        .arg("--require-changed-files")
        .arg("1")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"write_requested\": false"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"pre_verification\""))
        .stdout(predicate::str::contains("\"code\": \"preflight-gates\""))
        .stdout(predicate::str::contains("\"post_verification\": null"));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged dry-run fixture"),
        original
    );
}

#[test]
fn cli_fails_workspace_refactor_execute_before_write_on_policy_violation() {
    let dir = fresh_temp_dir("workspace-refactor-execute-target-conflict");
    let file = dir.join("core.lisp");
    let original = "(defun old-name (x) (new-name x))\n";
    fs::write(&file, original).expect("write workspace refactor execute fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-execute")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("symbol")
        .arg("--write")
        .arg("--fail-on-target-conflict")
        .arg("--require-changed-files")
        .arg("1")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "\"command\": \"workspace-refactor-execute\"",
        ))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains("\"target_occurrence_count\": 1"))
        .stdout(predicate::str::contains(
            "--fail-on-target-conflict found 1 existing replacement symbol occurrence(s)",
        ))
        .stderr(predicate::str::contains(
            "workspace-refactor-execute policy failed",
        ));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged policy failure fixture"),
        original
    );
}

#[test]
fn cli_fails_workspace_refactor_execute_before_write_on_preflight_violation() {
    let dir = fresh_temp_dir("workspace-refactor-execute-preflight");
    let file = dir.join("core.lisp");
    let original =
        "(defun old-name (x) x)\n(defun old-name (y) y)\n(defun caller () (old-name 1))\n";
    fs::write(&file, original).expect("write workspace refactor execute fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-execute")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("symbol")
        .arg("--write")
        .arg("--fail-on-no-change")
        .arg("--require-changed-files")
        .arg("1")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "\"command\": \"workspace-refactor-execute\"",
        ))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"pre_verification\""))
        .stdout(predicate::str::contains("\"phase\": \"pre\""))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "\"code\": \"ambiguous-definition\"",
        ))
        .stdout(predicate::str::contains("\"post_verification\": null"))
        .stderr(predicate::str::contains(
            "workspace-refactor-execute preflight failed",
        ));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged preflight failure fixture"),
        original
    );
}

#[test]
fn cli_fails_refactor_plan_policy_after_printing_json() {
    let dir = fresh_temp_dir("refactor-plan-policy");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    )
    .expect("write refactor plan policy fixture");

    let mut cmd = paredit();
    cmd.arg("refactor-plan")
        .arg("--symbol")
        .arg("render-pane")
        .arg("--operation")
        .arg("rename")
        .arg("--fail-on-blocking-gate")
        .arg("--require-definitions")
        .arg("2")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"policy\""))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains("\"fail_on_blocking_gate\": true"))
        .stdout(predicate::str::contains(
            "--require-definitions expected at least 2, found 1",
        ))
        .stderr(predicate::str::contains("refactor-plan policy failed"));
}

#[test]
fn cli_verifies_post_rename_refactor_invariants_for_agents() {
    let dir = fresh_temp_dir("verify-refactor");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun paint-pane (pane) (draw-pane pane))
(defun caller () (paint-pane window))
"#,
    )
    .expect("write verify refactor fixture");

    let mut cmd = paredit();
    cmd.arg("verify-refactor")
        .arg("--symbol")
        .arg("render-pane")
        .arg("--new-symbol")
        .arg("paint-pane")
        .arg("--operation")
        .arg("rename")
        .arg("--phase")
        .arg("post")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"rename\""))
        .stdout(predicate::str::contains("\"phase\": \"post\""))
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""))
        .stdout(predicate::str::contains(
            "\"code\": \"new-symbol-signature-compatible\"",
        ));
}
