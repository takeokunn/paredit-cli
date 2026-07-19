use super::*;

#[test]
fn cli_checks_refactor_manifest_without_writing_or_diffing() {
    let dir = fresh_temp_dir("refactor check");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");
    let canonical_root = fs::canonicalize(&dir).expect("canonicalize refactor root");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "preview"])
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

    let mut check = paredit();
    check
        .args(["refactor", "check"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--root")
        .arg(&dir)
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"root\": {"))
        .stdout(predicate::str::contains("\"enforced\": true"))
        .stdout(predicate::str::contains(
            canonical_root.display().to_string(),
        ))
        .stdout(predicate::str::contains("\"manifest_policy_passed\": true"))
        .stdout(predicate::str::contains("\"manifest_outputs_parse\": true"))
        .stdout(predicate::str::contains("\"status\": \"ready\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"run_refactor_diff_then_refactor_apply_write\"",
        ))
        .stdout(predicate::str::contains("\"blocked_reasons\": []"))
        .stdout(predicate::str::contains("\"steps\": ["))
        .stdout(predicate::str::contains("\"name\": \"manifest-policy\""))
        .stdout(predicate::str::contains("\"name\": \"apply-write\""))
        .stdout(predicate::str::contains("\"status\": \"scheduled\""))
        .stdout(predicate::str::contains("\"decision_summary\": {"))
        .stdout(predicate::str::contains("\"passed_step_count\": 4"))
        .stdout(predicate::str::contains("\"failed_step_count\": 0"))
        .stdout(predicate::str::contains("\"skipped_step_count\": 0"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 1"))
        .stdout(predicate::str::contains("\"blocked_reason_count\": 0"))
        .stdout(predicate::str::contains("\"can_apply\": true"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("\"stale_file_count\": 0"))
        .stdout(predicate::str::contains("\"input_hash_matches\": true"))
        .stdout(predicate::str::contains("\"output_hash_matches\": true"))
        .stdout(predicate::str::contains("\"stale\": false"))
        .stdout(predicate::str::contains("\"diff\"").not());

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read check fixture"),
        original
    );
}

#[test]
fn cli_refactor_manifest_round_trips_file_dialect() {
    let cases = [
        (
            "common-lisp-character-literal",
            "lisp",
            "common-lisp",
            "(old-name #\\))\n",
        ),
        (
            "emacs-lisp-character-literal",
            "el",
            "emacs-lisp",
            "(old-name ?\\))\n",
        ),
    ];

    for (case_name, extension, expected_dialect, original) in cases {
        let dir = fresh_temp_dir(case_name);
        let source = dir.join(format!("source.{extension}"));
        let manifest_file = dir.join("rename.preview.json");
        fs::write(&source, original).expect("write dialect fixture");

        let preview_output = paredit()
            .args([
                "refactor",
                "preview",
                "--from",
                "old-name",
                "--to",
                "new-name",
                "--mode",
                "symbol",
                "--fail-on-parse-error",
                "--output",
                "json",
            ])
            .arg(&source)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let manifest: serde_json::Value =
            serde_json::from_slice(&preview_output).expect("preview output parses");
        assert_eq!(
            manifest["files"][0]["dialect"].as_str(),
            Some(expected_dialect)
        );
        fs::write(&manifest_file, preview_output).expect("write refactor manifest");

        for command in ["check", "diff"] {
            paredit()
                .args(["refactor", command])
                .arg("--manifest")
                .arg(&manifest_file)
                .arg("--root")
                .arg(&dir)
                .arg("--output")
                .arg("json")
                .assert()
                .success()
                .stdout(predicate::str::contains("\"can_apply\": true"));
        }

        paredit()
            .args(["refactor", "apply"])
            .arg("--manifest")
            .arg(&manifest_file)
            .arg("--root")
            .arg(&dir)
            .arg("--write")
            .arg("--output")
            .arg("json")
            .assert()
            .success();

        assert_eq!(
            fs::read_to_string(&source).expect("read rewritten dialect fixture"),
            original.replace("old-name", "new-name")
        );
    }
}

#[test]
fn cli_refactor_manifest_requires_valid_file_dialect() {
    let dir = fresh_temp_dir("refactor manifest dialect validation");
    let source = dir.join("core.lisp");
    fs::write(&source, "(old-name value)\n").expect("write dialect validation fixture");
    let preview_output = paredit()
        .args([
            "refactor", "preview", "--from", "old-name", "--to", "new-name", "--mode", "symbol",
            "--output", "json",
        ])
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let manifest: serde_json::Value =
        serde_json::from_slice(&preview_output).expect("preview output parses");

    for (case_name, dialect, expected_error) in [
        (
            "missing",
            None,
            "missing required manifest field files[0].dialect",
        ),
        (
            "invalid",
            Some("not-a-dialect"),
            "manifest field files[0].dialect has invalid dialect",
        ),
    ] {
        let mut malformed = manifest.clone();
        let file = malformed["files"][0]
            .as_object_mut()
            .expect("manifest file entry");
        match dialect {
            Some(label) => {
                file.insert("dialect".into(), serde_json::Value::String(label.into()));
            }
            None => {
                file.remove("dialect");
            }
        }

        let manifest_file = dir.join(format!("{case_name}.preview.json"));
        fs::write(
            &manifest_file,
            serde_json::to_vec_pretty(&malformed).expect("serialize malformed manifest"),
        )
        .expect("write malformed manifest");

        paredit()
            .args(["refactor", "check"])
            .arg("--manifest")
            .arg(&manifest_file)
            .arg("--root")
            .arg(&dir)
            .assert()
            .failure()
            .stderr(predicate::str::contains(expected_error));
    }
}

#[test]
fn cli_refactor_check_rejects_unexpected_manifest_hash_without_writing() {
    let dir = fresh_temp_dir("refactor check-manifest-hash");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
        .arg("--fail-on-parse-error")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    fs::write(&manifest_file, preview_output).expect("write refactor manifest");

    let mut check = paredit();
    check
        .args(["refactor", "check"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--expect-manifest-hash")
        .arg("fnv1a64:0000000000000000")
        .arg("--root")
        .arg(&dir)
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("manifest hash mismatch"))
        .stderr(predicate::str::contains(
            "expected fnv1a64:0000000000000000",
        ))
        .stderr(predicate::str::contains("found fnv1a64:"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read hash mismatch fixture"),
        original
    );
}

#[test]
fn cli_refactor_check_reports_stale_manifest_without_writing() {
    let dir = fresh_temp_dir("refactor check-stale");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "preview"])
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

    let mut check = paredit();
    check
        .args(["refactor", "check"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"status\": \"blocked\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"regenerate_refactor_preview\"",
        ))
        .stdout(predicate::str::contains("\"stale_files\""))
        .stdout(predicate::str::contains("\"name\": \"file-freshness\""))
        .stdout(predicate::str::contains("\"status\": \"failed\""))
        .stdout(predicate::str::contains("\"name\": \"apply-write\""))
        .stdout(predicate::str::contains("\"status\": \"skipped\""))
        .stdout(predicate::str::contains("\"decision_summary\": {"))
        .stdout(predicate::str::contains("\"passed_step_count\": 2"))
        .stdout(predicate::str::contains("\"failed_step_count\": 2"))
        .stdout(predicate::str::contains("\"skipped_step_count\": 1"))
        .stdout(predicate::str::contains("\"scheduled_step_count\": 0"))
        .stdout(predicate::str::contains("\"blocked_reason_count\": 4"))
        .stdout(predicate::str::contains("\"can_apply\": false"))
        .stdout(predicate::str::contains("\"stale_file_count\": 1"))
        .stdout(predicate::str::contains("\"input_hash_matches\": false"))
        .stdout(predicate::str::contains("\"stale\": true"))
        .stderr(predicate::str::contains("refactor check validation failed"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read stale check fixture"),
        stale
    );
}

#[test]
fn cli_reports_output_hash_mismatch_refactor_check_manifest_without_writing() {
    let dir = fresh_temp_dir("refactor check-output-hash-mismatch");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "preview"])
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
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&preview_output).expect("preview output parses");
    manifest["files"][0]["output_hash"] = serde_json::Value::String("fnv1a64:deadbeef".into());
    fs::write(
        &manifest_file,
        serde_json::to_vec_pretty(&manifest).expect("serialize mismatched manifest"),
    )
    .expect("write mismatched manifest");

    let mut check = paredit();
    check
        .args(["refactor", "check"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
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
        .stderr(predicate::str::contains("output_hash_mismatches=1"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged hash mismatch fixture"),
        original
    );
}

#[test]
fn cli_reports_manifest_flag_mismatch_refactor_check_manifest_without_writing() {
    let dir = fresh_temp_dir("refactor check-manifest-flag-mismatch");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "preview"])
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
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&preview_output).expect("preview output parses");
    manifest["files"][0]["changed"] = serde_json::Value::Bool(false);
    fs::write(
        &manifest_file,
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest flag mismatch"),
    )
    .expect("write manifest flag mismatch");

    let mut check = paredit();
    check
        .args(["refactor", "check"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
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
        .stderr(predicate::str::contains("manifest_flag_mismatches=1"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged manifest flag fixture"),
        original
    );
}

#[test]
fn cli_refactor_check_rejects_manifest_paths_outside_root_without_writing() {
    let dir = fresh_temp_dir("refactor check-root-guard");
    let root = dir.join("workspace");
    let outside_file = dir.join("outside.lisp");
    let manifest_file = dir.join("rename.preview.json");
    fs::create_dir_all(&root).expect("create guarded workspace");
    let original = "(defun old-name (x) x)\n";
    fs::write(&outside_file, original).expect("write outside fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
        .arg("--fail-on-parse-error")
        .arg("--output")
        .arg("json")
        .arg(&outside_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    fs::write(&manifest_file, preview_output).expect("write refactor manifest");

    let mut check = paredit();
    check
        .args(["refactor", "check"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--root")
        .arg(&root)
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("outside refactor root"));

    assert_eq!(
        fs::read_to_string(&outside_file).expect("read unchanged outside fixture"),
        original
    );
}
