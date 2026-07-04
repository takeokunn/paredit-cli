use super::*;

#[test]
fn cli_applies_refactor_preview_manifest_with_hash_guards() {
    let dir = fresh_temp_dir("refactor-apply");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";
    fs::write(&lisp_file, original).expect("write lisp fixture");
    let canonical_root = fs::canonicalize(&dir).expect("canonicalize refactor root");

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

    let mut dry_run = paredit();
    dry_run
        .arg("refactor-apply")
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
        .stdout(predicate::str::contains("\"root\": {"))
        .stdout(predicate::str::contains("\"enforced\": true"))
        .stdout(predicate::str::contains(
            canonical_root.display().to_string(),
        ))
        .stdout(predicate::str::contains("\"hash\": \"fnv1a64:"))
        .stdout(predicate::str::contains(manifest_hash.clone()))
        .stdout(predicate::str::contains("\"write_requested\": false"))
        .stdout(predicate::str::contains("\"applied\": false"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"stale_file_count\": 0"))
        .stdout(predicate::str::contains(
            "\"output_hash_mismatch_count\": 0",
        ))
        .stdout(predicate::str::contains(
            "\"manifest_flag_mismatch_count\": 0",
        ))
        .stdout(predicate::str::contains("\"input_hash_matches\": true"))
        .stdout(predicate::str::contains("\"output_hash_matches\": true"))
        .stdout(predicate::str::contains("\"written\": false"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read dry-run fixture"),
        original
    );

    let mut apply = paredit();
    apply
        .arg("refactor-apply")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--expect-manifest-hash")
        .arg(&manifest_hash)
        .arg("--root")
        .arg(&dir)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"enforced\": true"))
        .stdout(predicate::str::contains("\"write_requested\": true"))
        .stdout(predicate::str::contains("\"applied\": true"))
        .stdout(predicate::str::contains("\"written_file_count\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten fixture"),
        "(defun new-name (x) x)\n(defun caller () (new-name 1) old-name)\n"
    );
}

#[test]
fn cli_refactor_apply_rejects_manifest_paths_outside_root_without_writing() {
    let dir = fresh_temp_dir("refactor-apply-root-guard");
    let root = dir.join("workspace");
    let outside_file = dir.join("outside.lisp");
    let manifest_file = dir.join("rename.preview.json");
    fs::create_dir_all(&root).expect("create guarded workspace");
    let original = "(defun old-name (x) x)\n";
    fs::write(&outside_file, original).expect("write outside fixture");

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
        .arg("--output")
        .arg("json")
        .arg(&outside_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    fs::write(&manifest_file, preview_output).expect("write refactor manifest");

    let mut apply = paredit();
    apply
        .arg("refactor-apply")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--root")
        .arg(&root)
        .arg("--write")
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

#[test]
fn cli_refuses_stale_refactor_apply_manifest_without_writing() {
    let dir = fresh_temp_dir("refactor-apply-stale");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    fs::write(
        &lisp_file,
        "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n",
    )
    .expect("write lisp fixture");

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

    let mut apply = paredit();
    apply
        .arg("refactor-apply")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"applied\": false"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"stale_file_count\": 1"))
        .stdout(predicate::str::contains("\"input_hash_matches\": false"))
        .stderr(predicate::str::contains("refactor-apply validation failed"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read stale fixture"),
        stale
    );
}
