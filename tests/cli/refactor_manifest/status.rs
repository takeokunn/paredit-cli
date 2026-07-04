use super::*;

#[test]
fn cli_refactor_status_reports_ready_write_plan_without_writing() {
    let dir = fresh_temp_dir("refactor-status-ready");
    let canonical_root = fs::canonicalize(&dir).expect("canonicalize status root");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut preview = paredit();
    let preview_output = preview
        .arg("refactor-preview")
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
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let manifest_text = String::from_utf8(preview_output).expect("preview output is utf8");
    let manifest_hash = stable_manifest_hash(&manifest_text);
    fs::write(&manifest_file, &manifest_text).expect("write refactor manifest");

    let mut status = paredit();
    status
        .arg("refactor-status")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--expect-manifest-hash")
        .arg(&manifest_hash)
        .arg("--root")
        .arg(&dir)
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ready\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"run_refactor_diff_then_refactor_apply_write\"",
        ))
        .stdout(predicate::str::contains("\"blocked_reasons\": []"))
        .stdout(predicate::str::contains("\"path\":"))
        .stdout(predicate::str::contains(
            manifest_file.display().to_string(),
        ))
        .stdout(predicate::str::contains(manifest_hash))
        .stdout(predicate::str::contains("\"can_apply\": true"))
        .stdout(predicate::str::contains("\"write_plan\": ["))
        .stdout(predicate::str::contains("\"edit_count\": 2"))
        .stdout(predicate::str::contains(
            canonical_root.display().to_string(),
        ))
        .stdout(predicate::str::contains("\"stale\": false"))
        .stdout(predicate::str::contains("\"diff\"").not());

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read status fixture"),
        original
    );
}

#[test]
fn cli_refactor_status_reports_blocked_stale_manifest_without_writing() {
    let dir = fresh_temp_dir("refactor-status-stale");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut preview = paredit();
    let preview_output = preview
        .arg("refactor-preview")
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
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    fs::write(&manifest_file, preview_output).expect("write refactor manifest");

    let stale = "(defun old-name (x) (+ x 1))\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, stale).expect("mutate lisp fixture after preview");

    let mut status = paredit();
    status
        .arg("refactor-status")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"blocked\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"regenerate_refactor_preview\"",
        ))
        .stdout(predicate::str::contains("\"stale_files\""))
        .stdout(predicate::str::contains("\"can_apply\": false"))
        .stdout(predicate::str::contains("\"stale_file_count\": 1"))
        .stdout(predicate::str::contains("\"write_plan\": []"))
        .stdout(predicate::str::contains("\"input_hash_matches\": false"))
        .stdout(predicate::str::contains("\"stale\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read stale status fixture"),
        stale
    );
}
