use super::*;

#[test]
fn cli_refactor_status_reports_blocked_stale_manifest_without_writing() {
    let dir = fresh_temp_dir("refactor status-stale");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    fs::write(&lisp_file, FUNCTION_FIXTURE).expect("write lisp fixture");
    write_manifest_fixture(
        &manifest_file,
        &String::from_utf8(preview_function_manifest(&lisp_file, true)).expect("preview utf8"),
    );

    let stale = "(defun old-name (x) (+ x 1))\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, stale).expect("mutate lisp fixture after preview");

    let mut status = paredit();
    status
        .args(["refactor", "status"])
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
        .stdout(predicate::str::contains("\"decision_summary\": {"))
        .stdout(predicate::str::contains("\"passed_step_count\": 2"))
        .stdout(predicate::str::contains("\"failed_step_count\": 2"))
        .stdout(predicate::str::contains("\"skipped_step_count\": 1"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 0"))
        .stdout(predicate::str::contains("\"blocked_reason_count\": 4"))
        .stdout(predicate::str::contains("\"can_apply\": false"))
        .stdout(predicate::str::contains("\"stale_file_count\": 1"))
        .stdout(predicate::str::contains("\"write_plan\": []"))
        .stdout(predicate::str::contains("\"input_hash_matches\": false"))
        .stdout(predicate::str::contains("\"stale\": true"));

    assert_status_preserves_file(&lisp_file, stale, "stale status fixture");
}

#[test]
fn cli_refactor_status_reports_policy_failed_manifest_without_writing() {
    let dir = fresh_temp_dir("refactor status-policy-failed");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let manifest_text = String::from_utf8(preview_function_manifest(&lisp_file, false))
        .expect("preview utf8")
        .replace("\"passed\": true", "\"passed\": false");
    write_manifest_fixture(&manifest_file, &manifest_text);

    let mut status = paredit();
    status
        .args(["refactor", "status"])
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
        .stdout(predicate::str::contains(
            "\"manifest_policy_passed\": false",
        ))
        .stdout(predicate::str::contains("\"manifest_outputs_parse\": true"))
        .stdout(predicate::str::contains("\"manifest_policy_failed\""))
        .stdout(predicate::str::contains("\"can_apply\": false"))
        .stdout(predicate::str::contains("\"write_plan\": []"));

    assert_status_preserves_file(&lisp_file, original, "policy failed status fixture");
}

#[test]
fn cli_refactor_status_reports_output_hash_mismatch_without_writing() {
    let dir = fresh_temp_dir("refactor status-output-hash-mismatch");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    fs::write(&lisp_file, FUNCTION_FIXTURE).expect("write lisp fixture");

    let mut manifest: serde_json::Value =
        serde_json::from_slice(&preview_function_manifest(&lisp_file, true))
            .expect("preview output parses");
    manifest["files"][0]["output_hash"] = serde_json::Value::String("fnv1a64:deadbeef".into());
    write_manifest_fixture(
        &manifest_file,
        &String::from_utf8(
            serde_json::to_vec_pretty(&manifest).expect("serialize mismatched manifest"),
        )
        .expect("manifest utf8"),
    );

    let mut status = paredit();
    status
        .args(["refactor", "status"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"blocked\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"fix_manifest_or_parser\"",
        ))
        .stdout(predicate::str::contains("\"output_hash_mismatches\""))
        .stdout(predicate::str::contains(
            "\"output_hash_mismatch_count\": 1",
        ))
        .stdout(predicate::str::contains("\"output_hash_matches\": false"))
        .stdout(predicate::str::contains("\"can_apply\": false"))
        .stdout(predicate::str::contains("\"write_plan\": []"));

    assert_status_preserves_file(&lisp_file, FUNCTION_FIXTURE, "hash mismatch fixture");
}

#[test]
fn cli_refactor_status_prioritizes_regenerate_preview_when_stale_and_output_hash_mismatch() {
    let dir = fresh_temp_dir("refactor status-stale-and-output-hash-mismatch");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    fs::write(&lisp_file, FUNCTION_FIXTURE).expect("write lisp fixture");

    let mut manifest: serde_json::Value =
        serde_json::from_slice(&preview_function_manifest(&lisp_file, true))
            .expect("preview output parses");
    manifest["files"][0]["output_hash"] = serde_json::Value::String("fnv1a64:deadbeef".into());
    write_manifest_fixture(
        &manifest_file,
        &String::from_utf8(
            serde_json::to_vec_pretty(&manifest).expect("serialize mismatched manifest"),
        )
        .expect("manifest utf8"),
    );
    let stale = "(defun old-name (x) (+ x 1))\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, stale).expect("mutate lisp fixture after preview");

    let mut status = paredit();
    status
        .args(["refactor", "status"])
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
        .stdout(predicate::str::contains("\"output_hash_mismatches\""))
        .stdout(predicate::str::contains("\"parse_errors\""))
        .stdout(predicate::str::contains("\"manifest_flag_mismatches\""))
        .stdout(predicate::str::contains("\"blocked_reason_count\": 4"))
        .stdout(predicate::str::contains("\"stale_file_count\": 1"))
        .stdout(predicate::str::contains(
            "\"output_hash_mismatch_count\": 1",
        ))
        .stdout(predicate::str::contains("\"parse_error_count\": 1"))
        .stdout(predicate::str::contains(
            "\"manifest_flag_mismatch_count\": 1",
        ))
        .stdout(predicate::str::contains("\"can_apply\": false"))
        .stdout(predicate::str::contains("\"write_plan\": []"));

    assert_status_preserves_file(&lisp_file, stale, "stale mismatch fixture");
}

#[test]
fn cli_refactor_status_reports_manifest_flag_mismatch_without_writing() {
    let dir = fresh_temp_dir("refactor status-manifest-flag-mismatch");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    fs::write(&lisp_file, FUNCTION_FIXTURE).expect("write lisp fixture");

    let mut manifest: serde_json::Value =
        serde_json::from_slice(&preview_function_manifest(&lisp_file, true))
            .expect("preview output parses");
    manifest["files"][0]["changed"] = serde_json::Value::Bool(false);
    write_manifest_fixture(
        &manifest_file,
        &String::from_utf8(
            serde_json::to_vec_pretty(&manifest).expect("serialize manifest flag mismatch"),
        )
        .expect("manifest utf8"),
    );

    let mut status = paredit();
    status
        .args(["refactor", "status"])
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
        .stdout(predicate::str::contains("\"manifest_flag_mismatches\""))
        .stdout(predicate::str::contains(
            "\"manifest_flag_mismatch_count\": 1",
        ))
        .stdout(predicate::str::contains("\"manifest_flags_match\": false"))
        .stdout(predicate::str::contains("\"can_apply\": false"))
        .stdout(predicate::str::contains("\"write_plan\": []"));

    assert_status_preserves_file(&lisp_file, FUNCTION_FIXTURE, "manifest flag fixture");
}

#[test]
fn cli_refactor_status_reports_unparseable_manifest_outputs_without_writing() {
    let dir = fresh_temp_dir("refactor status-unparseable-outputs");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let manifest_text = String::from_utf8(preview_function_manifest(&lisp_file, false))
        .expect("preview utf8")
        .replace(
            "\"all_outputs_parse\": true",
            "\"all_outputs_parse\": false",
        );
    write_manifest_fixture(&manifest_file, &manifest_text);

    let mut status = paredit();
    status
        .args(["refactor", "status"])
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
        .stdout(predicate::str::contains("\"manifest_policy_passed\": true"))
        .stdout(predicate::str::contains(
            "\"manifest_outputs_parse\": false",
        ))
        .stdout(predicate::str::contains(
            "\"manifest_outputs_do_not_parse\"",
        ))
        .stdout(predicate::str::contains("\"can_apply\": false"))
        .stdout(predicate::str::contains("\"write_plan\": []"));

    assert_status_preserves_file(&lisp_file, original, "unparseable-output status fixture");
}
