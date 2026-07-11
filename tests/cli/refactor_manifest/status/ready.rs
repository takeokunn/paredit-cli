use super::*;

#[test]
fn cli_refactor_status_reports_ready_write_plan_without_writing() {
    let dir = fresh_temp_dir("refactor status-ready");
    let canonical_root = fs::canonicalize(&dir).expect("canonicalize status root");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    fs::write(&lisp_file, FUNCTION_FIXTURE).expect("write lisp fixture");

    let manifest_text =
        String::from_utf8(preview_function_manifest(&lisp_file, true)).expect("preview utf8");
    let manifest_hash = stable_manifest_hash(&manifest_text);
    write_manifest_fixture(&manifest_file, &manifest_text);

    let mut status = paredit();
    status
        .args(["refactor", "status"])
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
        .stdout(predicate::str::contains("\"steps\": ["))
        .stdout(predicate::str::contains(
            "\"name\": \"manifest-outputs-parse\"",
        ))
        .stdout(predicate::str::contains("\"name\": \"apply-write\""))
        .stdout(predicate::str::contains("\"status\": \"scheduled\""))
        .stdout(predicate::str::contains("\"decision_summary\": {"))
        .stdout(predicate::str::contains("\"passed_step_count\": 4"))
        .stdout(predicate::str::contains("\"failed_step_count\": 0"))
        .stdout(predicate::str::contains("\"skipped_step_count\": 0"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 1"))
        .stdout(predicate::str::contains("\"blocked_reason_count\": 0"))
        .stdout(predicate::str::contains("\"path\":"))
        .stdout(predicate::str::contains(
            manifest_file.display().to_string(),
        ))
        .stdout(predicate::str::contains(manifest_hash))
        .stdout(predicate::str::contains("\"can_apply\": true"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("\"write_plan\": ["))
        .stdout(predicate::str::contains("\"edit_count\": 2"))
        .stdout(predicate::str::contains(
            canonical_root.display().to_string(),
        ))
        .stdout(predicate::str::contains("\"stale\": false"))
        .stdout(predicate::str::contains("\"diff\"").not());

    assert_status_preserves_file(&lisp_file, FUNCTION_FIXTURE, "ready status fixture");
}
