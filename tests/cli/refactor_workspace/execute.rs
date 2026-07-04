use super::*;

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
        .stdout(predicate::str::contains("\"run_pre_verification\": true"))
        .stdout(predicate::str::contains("\"apply_preview\": true"))
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
fn cli_executes_workspace_refactor_with_remove_operation_verification() {
    let dir = fresh_temp_dir("workspace-refactor-execute-remove-verification");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(defun stale-helper (x) x)\n(defun caller () (live-helper 1))\n",
    )
    .expect("write remove verification fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-execute")
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
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains("\"new-symbol-present\"").not());
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
        .stdout(predicate::str::contains("\"write_plan\""))
        .stdout(predicate::str::contains("\"write_allowed\": false"))
        .stdout(predicate::str::contains("\"writable_file_count\": 0"))
        .stdout(predicate::str::contains("\"writable_files\": []"))
        .stdout(predicate::str::contains("\"refused_file_count\": 0"))
        .stdout(predicate::str::contains("\"refused_files\": []"))
        .stdout(predicate::str::contains("\"refusal\": null"))
        .stdout(predicate::str::contains("\"preflight_decision\""))
        .stdout(predicate::str::contains("\"execute_decision\""))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains(
            "\"reason\": \"all-dry-run-gates-passed\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-preview-or-rerun-with-write\"",
        ))
        .stdout(predicate::str::contains("\"name\": \"post-verification\""))
        .stdout(predicate::str::contains("\"status\": \"skipped\""))
        .stdout(predicate::str::contains("\"run_post_verification\": false"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
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
        .stdout(predicate::str::contains("\"preflight_decision\""))
        .stdout(predicate::str::contains("\"execute_decision\""))
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
        .stdout(predicate::str::contains("\"preflight_decision\""))
        .stdout(predicate::str::contains("\"execute_decision\""))
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
