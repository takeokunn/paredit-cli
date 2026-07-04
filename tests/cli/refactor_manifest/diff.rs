use super::*;

#[test]
fn cli_prints_refactor_diff_from_manifest_without_writing() {
    let dir = fresh_temp_dir("refactor-diff");
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
    fs::write(&manifest_file, preview_output).expect("write refactor manifest");

    let mut diff = paredit();
    diff.arg("refactor-diff")
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
        .stdout(predicate::str::contains("\"can_apply\": true"))
        .stdout(predicate::str::contains("\"changed_file_count\": 1"))
        .stdout(predicate::str::contains("\"stale_file_count\": 0"))
        .stdout(predicate::str::contains("\"input_hash_matches\": true"))
        .stdout(predicate::str::contains("--- "))
        .stdout(predicate::str::contains("-"))
        .stdout(predicate::str::contains("old-name"))
        .stdout(predicate::str::contains("+"))
        .stdout(predicate::str::contains("new-name"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read diff fixture"),
        original
    );
}

#[test]
fn cli_refactor_diff_rejects_unexpected_manifest_hash_without_writing() {
    let dir = fresh_temp_dir("refactor-diff-manifest-hash");
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
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    fs::write(&manifest_file, preview_output).expect("write refactor manifest");

    let mut diff = paredit();
    diff.arg("refactor-diff")
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
fn cli_refactor_diff_rejects_manifest_paths_outside_root_without_writing() {
    let dir = fresh_temp_dir("refactor-diff-root-guard");
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

    let mut diff = paredit();
    diff.arg("refactor-diff")
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

#[test]
fn cli_refactor_diff_reports_stale_manifest_without_writing() {
    let dir = fresh_temp_dir("refactor-diff-stale");
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

    let mut diff = paredit();
    diff.arg("refactor-diff")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"can_apply\": false"))
        .stdout(predicate::str::contains("\"stale_file_count\": 1"))
        .stdout(predicate::str::contains("\"stale\": true"))
        .stderr(predicate::str::contains("refactor-diff validation failed"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read stale fixture"),
        stale
    );
}
