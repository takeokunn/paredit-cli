use super::*;

#[test]
fn cli_previews_function_refactor_without_writing_files() {
    let dir = fresh_temp_dir("refactor preview");
    let lisp_file = dir.join("core.lisp");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--max-preview-bytes")
        .arg("120")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"mode\": \"function\""))
        .stdout(predicate::str::contains("\"write_requested\": false"))
        .stdout(predicate::str::contains("\"write_plan\""))
        .stdout(predicate::str::contains("\"write_allowed\": false"))
        .stdout(predicate::str::contains("\"writable_file_count\": 0"))
        .stdout(predicate::str::contains("\"writable_files\": []"))
        .stdout(predicate::str::contains("\"refused_file_count\": 0"))
        .stdout(predicate::str::contains("\"refused_files\": []"))
        .stdout(predicate::str::contains("\"refusal\": null"))
        .stdout(predicate::str::contains("\"decision\""))
        .stdout(predicate::str::contains("\"status\": \"dry-run-ready\""))
        .stdout(predicate::str::contains(
            "\"reason\": \"all-dry-run-gates-passed\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-preview-or-rerun-with-write\"",
        ))
        .stdout(predicate::str::contains("\"apply_preview\": false"))
        .stdout(predicate::str::contains("\"name\": \"preview-policy\""))
        .stdout(predicate::str::contains("\"name\": \"write-output-parse\""))
        .stdout(predicate::str::contains("\"name\": \"apply-preview\""))
        .stdout(predicate::str::contains("\"status\": \"scheduled\""))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"edit_count\": 2"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"output_parse_ok\": true"))
        .stdout(predicate::str::contains("\"input_hash\": \"fnv1a64:"))
        .stdout(predicate::str::contains("\"edits\": ["))
        .stdout(predicate::str::contains("\"start\":"))
        .stdout(predicate::str::contains("\"end\":"))
        .stdout(predicate::str::contains("\"replacement\": \"new-name\""))
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains("\"violation_count\": 0"))
        .stdout(predicate::str::contains("\"write_blocked\": false"))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-preview-or-rerun-with-write\"",
        ))
        .stdout(predicate::str::contains("new-name"))
        .stdout(predicate::str::contains("old-name"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_writes_refactor_preview_after_policy_and_parse_gates() {
    let dir = fresh_temp_dir("refactor preview-write");
    let lisp_file = dir.join("core.lisp");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
        .arg("--fail-on-parse-error")
        .arg("--require-definitions")
        .arg("1")
        .arg("--require-edits")
        .arg("2")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"write_requested\": true"))
        .stdout(predicate::str::contains("\"write_allowed\": true"))
        .stdout(predicate::str::contains("\"writable_file_count\": 1"))
        .stdout(predicate::str::contains("\"writable_files\": ["))
        .stdout(predicate::str::contains("\"refused_file_count\": 0"))
        .stdout(predicate::str::contains("\"refused_files\": []"))
        .stdout(predicate::str::contains("\"refusal\": null"))
        .stdout(predicate::str::contains("\"status\": \"write-applied\""))
        .stdout(predicate::str::contains(
            "\"reason\": \"preview-write-applied\"",
        ))
        .stdout(predicate::str::contains(
            "\"next_action\": \"run-verification-or-review-diff\"",
        ))
        .stdout(predicate::str::contains("\"apply_preview\": true"))
        .stdout(predicate::str::contains("\"name\": \"apply-preview\""))
        .stdout(predicate::str::contains("\"status\": \"passed\""))
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
        fs::read_to_string(&lisp_file).expect("read rewritten fixture"),
        "(defun new-name (x) x)\n(defun caller () (new-name 1) old-name)\n"
    );
}

#[test]
fn cli_refuses_refactor_preview_write_when_policy_fails() {
    let dir = fresh_temp_dir("refactor preview-write-policy");
    let lisp_file = dir.join("core.lisp");
    let original = "(defun old-name (x) x)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--require-edits")
        .arg("3")
        .arg("--write")
        .arg(&lisp_file)
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
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains("\"violation_count\": 1"))
        .stdout(predicate::str::contains("\"write_blocked\": true"))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-policy-violations\"",
        ))
        .stderr(predicate::str::contains("refactor preview policy failed"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_fails_refactor_preview_definition_policy_after_printing_json() {
    let dir = fresh_temp_dir("refactor preview-definition-policy");
    let lisp_file = dir.join("core.lisp");
    let original = "(defun old-name (x) x)\n(defun old-name (y) y)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--require-definitions")
        .arg("1")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"definition_count\": 2"))
        .stdout(predicate::str::contains("\"require_definitions\": 1"))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "--require-definitions expected exactly 1, found 2",
        ))
        .stderr(predicate::str::contains("refactor preview policy failed"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_fails_refactor_preview_policy_after_printing_json() {
    let dir = fresh_temp_dir("refactor preview-policy");
    let lisp_file = dir.join("core.lisp");
    let original = "(defun old-name (x) x)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--require-edits")
        .arg("3")
        .arg("--fail-on-parse-error")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains("\"require_edits\": 3"))
        .stdout(predicate::str::contains(
            "--require-edits expected at least 3, found 1",
        ))
        .stderr(predicate::str::contains("refactor preview policy failed"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        original
    );
}

#[test]
fn cli_fails_refactor_preview_when_target_symbol_already_exists() {
    let dir = fresh_temp_dir("refactor preview-target-conflict");
    let lisp_file = dir.join("core.lisp");
    let original = "(defun old-name (x) (new-name x))\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("symbol")
        .arg("--fail-on-target-conflict")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"target_occurrence_count\": 1"))
        .stdout(predicate::str::contains(
            "\"fail_on_target_conflict\": true",
        ))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains(
            "--fail-on-target-conflict found 1 existing replacement symbol occurrence(s)",
        ))
        .stderr(predicate::str::contains("refactor preview policy failed"))
        .stderr(predicate::str::contains("--fail-on-target-conflict"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        original
    );
}

#[cfg(unix)]
#[test]
fn cli_rolls_back_refactor_preview_when_later_file_write_fails() {
    use std::os::unix::fs::PermissionsExt;

    let dir = fresh_temp_dir("refactor preview-rollback");
    let writable_file = dir.join("core.lisp");
    let readonly_dir = dir.join("readonly");
    let blocked_file = readonly_dir.join("other.lisp");
    let writable_original = "(defun old-name (x) x)\n";
    let blocked_original = "(defun caller () (old-name 1))\n";
    fs::write(&writable_file, writable_original).expect("write writable fixture");
    fs::create_dir_all(&readonly_dir).expect("create readonly dir");
    fs::write(&blocked_file, blocked_original).expect("write blocked fixture");
    fs::set_permissions(&readonly_dir, fs::Permissions::from_mode(0o555))
        .expect("chmod readonly dir");

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--write")
        .arg(&writable_file)
        .arg(&blocked_file)
        .assert();

    fs::set_permissions(&readonly_dir, fs::Permissions::from_mode(0o755))
        .expect("restore readonly dir permissions");

    assert
        .failure()
        .stderr(predicate::str::contains("Permission denied"));
    assert_eq!(
        fs::read_to_string(&writable_file).expect("read rolled back writable fixture"),
        writable_original
    );
    assert_eq!(
        fs::read_to_string(&blocked_file).expect("read blocked fixture"),
        blocked_original
    );
}
