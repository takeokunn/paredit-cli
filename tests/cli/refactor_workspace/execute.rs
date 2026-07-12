use super::*;

#[test]
fn cli_executes_workspace_refactor_with_post_verification() {
    let dir = fresh_temp_dir("workspace refactor-execute");
    let lisp_file = dir.join("src/core.lisp");
    let elisp_file = dir.join("src/ui.el");
    let ignored = dir.join("src/notes.txt");
    write_fixture(
        &lisp_file,
        "(defun old-name (x) (list x))\n(defun caller () (old-name 1))\n",
    );
    write_fixture(
        &elisp_file,
        "(defun ui () (old-name 2))\n(message \"old-name\")\n",
    );
    write_fixture(&ignored, "old-name is plain text");

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "workspace-execute"])
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
            "\"command\": \"refactor workspace-execute\"",
        ))
        .stdout(predicate::str::contains("\"mode\": \"symbol\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 2"))
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("ui.el"))
        .stdout(predicate::str::contains("\"written_file_count\": 2"))
        .stdout(predicate::str::contains("\"write_plan\""))
        .stdout(predicate::str::contains("\"write_allowed\": true"))
        .stdout(predicate::str::contains("\"writable_file_count\": 2"))
        .stdout(predicate::str::contains("\"writable_files\": ["))
        .stdout(predicate::str::contains("\"refused_file_count\": 0"))
        .stdout(predicate::str::contains("\"refused_files\": []"))
        .stdout(predicate::str::contains("\"refusal\": null"))
        .stdout(predicate::str::contains("\"preflight_decision\""))
        .stdout(predicate::str::contains("\"execute_decision\""))
        .stdout(predicate::str::contains("\"outcome\""))
        .stdout(predicate::str::contains("\"status\": \"ready-to-write\""))
        .stdout(predicate::str::contains(
            "\"reason\": \"all-execute-gates-passed\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"write-preview-and-run-post-verification\"",
        ))
        .stdout(predicate::str::contains("\"steps\""))
        .stdout(predicate::str::contains("\"name\": \"preview-policy\""))
        .stdout(predicate::str::contains("\"name\": \"post-verification\""))
        .stdout(predicate::str::contains("\"status\": \"scheduled\""))
        .stdout(predicate::str::contains("\"passed_step_count\": 3"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 2"))
        .stdout(predicate::str::contains("\"status\": \"write-applied\""))
        .stdout(predicate::str::contains(
            "\"reason\": \"write-and-post-verification-passed\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-written-files\"",
        ))
        .stdout(predicate::str::contains("\"summary\""))
        .stdout(predicate::str::contains("\"passed_step_count\": 5"))
        .stdout(predicate::str::contains("\"failed_step_count\": 0"))
        .stdout(predicate::str::contains("\"skipped_step_count\": 0"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 0"))
        .stdout(predicate::str::contains("\"write_applied\": true"))
        .stdout(predicate::str::contains(
            "\"post_verification_passed\": true",
        ))
        .stdout(predicate::str::contains("\"run_pre_verification\": true"))
        .stdout(predicate::str::contains("\"apply_preview\": true"))
        .stdout(predicate::str::contains("\"pre_verification\""))
        .stdout(predicate::str::contains("\"post_verification\""))
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains("\"target_kind\": \"callable\""))
        .stdout(predicate::str::contains("\"violation_count\": 0"))
        .stdout(predicate::str::contains("\"write_blocked\": false"))
        .stdout(predicate::str::contains("\"code\": \"preflight-gates\""))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""));

    let report = parse_cli_json(&assert.get_output().stdout);
    assert_eq!(
        report["pre_verification"]["target_kind"],
        serde_json::Value::String("callable".to_string())
    );
    assert_eq!(
        report["post_verification"]["target_kind"],
        serde_json::Value::String("callable".to_string())
    );

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
fn cli_executes_workspace_refactor_with_remove_operation_verification() {
    let dir = fresh_temp_dir("workspace refactor-execute-remove-verification");
    let file = dir.join("core.lisp");
    write_fixture(
        &file,
        "(defun stale-helper (x) x)\n(defun caller () (live-helper 1))\n",
    );

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "workspace-execute"])
        .arg("--from")
        .arg("stale-helper")
        .arg("--to")
        .arg("removed-helper")
        .arg("--operation")
        .arg("remove")
        .arg("--mode")
        .arg("symbol")
        .arg("--write")
        .arg("--fail-on-no-change")
        .arg("--require-changed-files")
        .arg("1")
        .arg("--require-edits")
        .arg("1")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"remove\""))
        .stdout(predicate::str::contains("\"phase\": \"post\""))
        .stdout(predicate::str::contains("\"target_kind\": \"callable\""))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains("\"new-symbol-present\"").not());

    let report = parse_cli_json(&assert.get_output().stdout);
    assert_eq!(
        report["post_verification"]["operation"],
        serde_json::Value::String("remove".to_string())
    );
    assert_eq!(
        report["post_verification"]["phase"],
        serde_json::Value::String("post".to_string())
    );
    assert_eq!(
        report["post_verification"]["target_kind"],
        serde_json::Value::String("callable".to_string())
    );
}

#[test]
fn cli_dry_runs_workspace_refactor_execute_without_writing() {
    let dir = fresh_temp_dir("workspace refactor-execute-dry-run");
    let file = dir.join("core.lisp");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1))\n";
    write_fixture(&file, original);

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "workspace-execute"])
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
        .stdout(predicate::str::contains("\"write_plan\""))
        .stdout(predicate::str::contains("\"write_allowed\": false"))
        .stdout(predicate::str::contains("\"writable_file_count\": 0"))
        .stdout(predicate::str::contains("\"writable_files\": []"))
        .stdout(predicate::str::contains("\"refused_file_count\": 0"))
        .stdout(predicate::str::contains("\"refused_files\": []"))
        .stdout(predicate::str::contains("\"refusal\": null"))
        .stdout(predicate::str::contains("\"preflight_decision\""))
        .stdout(predicate::str::contains("\"execute_decision\""))
        .stdout(predicate::str::contains("\"outcome\""))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains(
            "\"reason\": \"all-dry-run-gates-passed\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-preview-or-rerun-with-write\"",
        ))
        .stdout(predicate::str::contains("\"name\": \"post-verification\""))
        .stdout(predicate::str::contains("\"status\": \"skipped\""))
        .stdout(predicate::str::contains("\"passed_step_count\": 3"))
        .stdout(predicate::str::contains("\"failed_step_count\": 0"))
        .stdout(predicate::str::contains("\"skipped_step_count\": 1"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 1"))
        .stdout(predicate::str::contains("\"run_post_verification\": false"))
        .stdout(predicate::str::contains("\"write_applied\": false"))
        .stdout(predicate::str::contains(
            "\"post_verification_passed\": null",
        ))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"pre_verification\""))
        .stdout(predicate::str::contains("\"code\": \"preflight-gates\""))
        .stdout(predicate::str::contains("\"post_verification\": null"));

    let report = parse_cli_json(&assert.get_output().stdout);
    assert_eq!(
        report["preflight_decision"]["summary"]["run_pre_verification"],
        serde_json::Value::Bool(true)
    );
    assert_eq!(
        report["execute_decision"]["summary"]["run_post_verification"],
        serde_json::Value::Bool(false)
    );
    assert_eq!(
        report["execute_decision"]["summary"]["scheduled_step_count"],
        serde_json::Value::Number(1_u64.into())
    );
    assert_eq!(
        report["outcome"]["summary"]["write_applied"],
        serde_json::Value::Bool(false)
    );
    assert_eq!(
        report["outcome"]["summary"]["post_verification_passed"],
        serde_json::Value::Null
    );

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged dry-run fixture"),
        original
    );
}

#[test]
fn cli_fails_workspace_refactor_execute_before_write_on_policy_violation() {
    let dir = fresh_temp_dir("workspace refactor-execute-target-conflict");
    let file = dir.join("core.lisp");
    let original = "(defun old-name (x) (new-name x))\n";
    write_fixture(&file, original);

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-execute"])
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
            "\"command\": \"refactor workspace-execute\"",
        ))
        .stdout(predicate::str::contains("\"preflight_decision\""))
        .stdout(predicate::str::contains("\"execute_decision\""))
        .stdout(predicate::str::contains("\"outcome\""))
        .stdout(predicate::str::contains("\"write_plan\""))
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
        .stdout(predicate::str::contains("\"name\": \"preview-policy\""))
        .stdout(predicate::str::contains("\"status\": \"failed\""))
        .stdout(predicate::str::contains("\"name\": \"write-output-parse\""))
        .stdout(predicate::str::contains("\"status\": \"skipped\""))
        .stdout(predicate::str::contains("\"passed_step_count\": 0"))
        .stdout(predicate::str::contains("\"failed_step_count\": 1"))
        .stdout(predicate::str::contains("\"skipped_step_count\": 4"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 0"))
        .stdout(predicate::str::contains("\"write_applied\": false"))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains("\"violation_count\": 1"))
        .stdout(predicate::str::contains("\"write_blocked\": true"))
        .stdout(predicate::str::contains("\"target_occurrence_count\": 1"))
        .stdout(predicate::str::contains(
            "--fail-on-target-conflict found 1 existing replacement symbol occurrence(s)",
        ))
        .stderr(predicate::str::contains(
            "refactor workspace-execute policy failed",
        ));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged policy failure fixture"),
        original
    );
}

#[test]
fn cli_fails_workspace_refactor_execute_before_write_on_preflight_violation() {
    let dir = fresh_temp_dir("workspace refactor-execute-preflight");
    let file = dir.join("core.lisp");
    let original =
        "(defun old-name (x) x)\n(defun old-name (y) y)\n(defun caller () (old-name 1))\n";
    write_fixture(&file, original);

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "workspace-execute"])
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
            "\"command\": \"refactor workspace-execute\"",
        ))
        .stdout(predicate::str::contains("\"preflight_decision\""))
        .stdout(predicate::str::contains("\"execute_decision\""))
        .stdout(predicate::str::contains("\"outcome\""))
        .stdout(predicate::str::contains("\"write_plan\""))
        .stdout(predicate::str::contains("\"write_allowed\": true"))
        .stdout(predicate::str::contains("\"writable_file_count\": 1"))
        .stdout(predicate::str::contains("\"writable_files\": ["))
        .stdout(predicate::str::contains("\"refused_file_count\": 0"))
        .stdout(predicate::str::contains("\"refused_files\": []"))
        .stdout(predicate::str::contains("\"refusal\": null"))
        .stdout(predicate::str::contains(
            "\"status\": \"blocked-by-pre-verification\"",
        ))
        .stdout(predicate::str::contains(
            "\"reason\": \"pre-verification-failed\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-pre-verification-checks\"",
        ))
        .stdout(predicate::str::contains("\"name\": \"pre-verification\""))
        .stdout(predicate::str::contains("\"status\": \"failed\""))
        .stdout(predicate::str::contains("\"passed_step_count\": 2"))
        .stdout(predicate::str::contains("\"failed_step_count\": 1"))
        .stdout(predicate::str::contains("\"skipped_step_count\": 2"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 0"))
        .stdout(predicate::str::contains("\"write_applied\": false"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"pre_verification\""))
        .stdout(predicate::str::contains("\"phase\": \"pre\""))
        .stdout(predicate::str::contains("\"target_kind\": \"callable\""))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "\"code\": \"ambiguous-definition\"",
        ))
        .stdout(predicate::str::contains("\"post_verification\": null"))
        .stderr(predicate::str::contains(
            "refactor workspace-execute preflight failed",
        ));

    let report = parse_cli_json(&assert.get_output().stdout);
    assert_eq!(
        report["pre_verification"]["phase"],
        serde_json::Value::String("pre".to_string())
    );
    assert_eq!(
        report["pre_verification"]["target_kind"],
        serde_json::Value::String("callable".to_string())
    );
    assert_eq!(report["post_verification"], serde_json::Value::Null);

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged preflight failure fixture"),
        original
    );
}

#[test]
fn cli_refuses_workspace_refactor_execute_write_when_rewritten_output_does_not_parse() {
    let dir = fresh_temp_dir("workspace refactor-execute-write-parse-refused");
    let file = dir.join("core.lisp");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1))\n";
    write_fixture(&file, original);

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-execute"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("#|")
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
            "\"command\": \"refactor workspace-execute\"",
        ))
        .stdout(predicate::str::contains("\"preflight_decision\""))
        .stdout(predicate::str::contains("\"execute_decision\""))
        .stdout(predicate::str::contains("\"outcome\""))
        .stdout(predicate::str::contains("\"write_plan\""))
        .stdout(predicate::str::contains("\"write_allowed\": false"))
        .stdout(predicate::str::contains("\"writable_file_count\": 0"))
        .stdout(predicate::str::contains("\"writable_files\": []"))
        .stdout(predicate::str::contains("\"refused_file_count\": 1"))
        .stdout(predicate::str::contains("\"refused_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("\"refusal\": {"))
        .stdout(predicate::str::contains(
            "\"status\": \"unparsable-outputs\"",
        ))
        .stdout(predicate::str::contains(
            "\"status\": \"refused-unparsable-output\"",
        ))
        .stdout(predicate::str::contains(
            "\"reason\": \"rewritten-output-did-not-parse\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"inspect-preview-parse-errors\"",
        ))
        .stdout(predicate::str::contains("\"name\": \"write-output-parse\""))
        .stdout(predicate::str::contains("\"status\": \"failed\""))
        .stdout(predicate::str::contains("\"name\": \"pre-verification\""))
        .stdout(predicate::str::contains("\"status\": \"skipped\""))
        .stdout(predicate::str::contains("\"passed_step_count\": 1"))
        .stdout(predicate::str::contains("\"failed_step_count\": 1"))
        .stdout(predicate::str::contains("\"skipped_step_count\": 3"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 0"))
        .stdout(predicate::str::contains("\"write_parse_refused\": true"))
        .stdout(predicate::str::contains("\"run_pre_verification\": false"))
        .stdout(predicate::str::contains("\"apply_preview\": false"))
        .stdout(predicate::str::contains("\"run_post_verification\": false"))
        .stdout(predicate::str::contains("\"write_applied\": false"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"pre_verification\": null"))
        .stdout(predicate::str::contains("\"post_verification\": null"))
        .stdout(predicate::str::contains("\"output_parse_ok\": false"))
        .stderr(predicate::str::contains(
            "refactor workspace-execute write refused because rewritten output failed to parse",
        ));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged parse refusal fixture"),
        original
    );
}

#[test]
fn cli_fails_workspace_refactor_execute_after_write_on_post_verification_violation() {
    let dir = fresh_temp_dir("workspace refactor-execute-post-verification");
    let file = dir.join("core.lisp");
    let original =
        "(defun old-name (x) x)\n(defun new-name (y) y)\n(defun caller () (old-name 1))\n";
    write_fixture(&file, original);

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "workspace-execute"])
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
            "\"command\": \"refactor workspace-execute\"",
        ))
        .stdout(predicate::str::contains("\"preflight_decision\""))
        .stdout(predicate::str::contains("\"execute_decision\""))
        .stdout(predicate::str::contains("\"outcome\""))
        .stdout(predicate::str::contains("\"status\": \"ready-to-write\""))
        .stdout(predicate::str::contains(
            "\"status\": \"post-verification-failed\"",
        ))
        .stdout(predicate::str::contains(
            "\"reason\": \"post-verification-failed\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-post-verification-checks\"",
        ))
        .stdout(predicate::str::contains("\"write_applied\": true"))
        .stdout(predicate::str::contains(
            "\"post_verification_passed\": false",
        ))
        .stdout(predicate::str::contains("\"written_file_count\": 1"))
        .stdout(predicate::str::contains("\"pre_verification\""))
        .stdout(predicate::str::contains("\"post_verification\""))
        .stdout(predicate::str::contains("\"phase\": \"post\""))
        .stdout(predicate::str::contains("\"target_kind\": \"callable\""))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "\"code\": \"new-symbol-signature-compatible\"",
        ))
        .stderr(predicate::str::contains(
            "refactor workspace-execute post verification failed",
        ));

    let report = parse_cli_json(&assert.get_output().stdout);
    assert_eq!(
        report["pre_verification"]["target_kind"],
        serde_json::Value::String("callable".to_string())
    );
    assert_eq!(
        report["post_verification"]["phase"],
        serde_json::Value::String("post".to_string())
    );
    assert_eq!(
        report["post_verification"]["target_kind"],
        serde_json::Value::String("callable".to_string())
    );

    assert_eq!(
        fs::read_to_string(&file).expect("read post verification failure fixture"),
        "(defun new-name (x) x)\n(defun new-name (y) y)\n(defun caller () (new-name 1))\n"
    );
}

#[cfg(unix)]
#[test]
fn cli_reports_skipped_symlink_in_workspace_refactor_execute_dry_run() {
    let dir = fresh_temp_dir("workspace refactor-execute-symlink");
    let file = dir.join("src/core.lisp");
    let symlink_path = dir.join("linked-core.lisp");
    write_fixture(
        &file,
        "(defun old-name (x) x)\n(defun caller () (old-name 1))\n",
    );
    std::os::unix::fs::symlink(&file, &symlink_path).expect("create workspace symlink");

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-execute"])
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
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 1"))
        .stdout(predicate::str::contains("\"symlink\": 1"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"write_requested\": false"))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains("core.lisp"));
}

#[test]
fn cli_dry_runs_workspace_refactor_execute_with_hidden_and_generated_inputs() {
    let dir = fresh_temp_dir("workspace refactor-execute-discovery-flags");
    let main_file = dir.join("src/core.lisp");
    let hidden_file = dir.join(".hidden/secret.lisp");
    let generated_file = dir.join("target/generated.lisp");
    write_fixture(
        &main_file,
        "(defun render-pane (pane) pane)\n(defun caller () (render-pane window))\n",
    );
    write_fixture(
        &hidden_file,
        "(defun hidden-caller () (render-pane hidden-window))\n",
    );
    write_fixture(
        &generated_file,
        "(defun generated-caller () (render-pane generated-window))\n",
    );

    let mut cmd = paredit();
    cmd.args(["refactor", "workspace-execute"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--include-hidden")
        .arg("--include-generated")
        .arg("--fail-on-no-change")
        .arg("--require-changed-files")
        .arg("3")
        .arg("--require-definitions")
        .arg("1")
        .arg("--require-edits")
        .arg("4")
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
        .stdout(predicate::str::contains("\"changed_file_count\": 3"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"write_requested\": false"))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains("\"edit_count\": 4"))
        .stdout(predicate::str::contains("\"violation_count\": 0"))
        .stdout(predicate::str::contains("\"write_blocked\": false"))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("secret.lisp"))
        .stdout(predicate::str::contains("generated.lisp"));
}

#[test]
fn cli_dry_runs_workspace_refactor_execute_with_unknown_inputs() {
    let dir = fresh_temp_dir("workspace refactor-execute-unknown");
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
    cmd.args(["refactor", "workspace-execute"])
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
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"write_requested\": false"))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains("\"edit_count\": 3"))
        .stdout(predicate::str::contains("\"violation_count\": 0"))
        .stdout(predicate::str::contains("\"write_blocked\": false"))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("scratch.txt"));
}
