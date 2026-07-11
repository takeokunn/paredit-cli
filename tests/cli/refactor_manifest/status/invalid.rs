use super::*;

#[test]
fn cli_refactor_status_rejects_unexpected_manifest_hash_without_writing() {
    let dir = fresh_temp_dir("refactor status-manifest-hash");
    let lisp_file = dir.join("core.lisp");
    let manifest_file = dir.join("rename.preview.json");
    fs::write(&lisp_file, FUNCTION_FIXTURE).expect("write lisp fixture");
    write_manifest_fixture(
        &manifest_file,
        &String::from_utf8(preview_function_manifest(&lisp_file, false)).expect("preview utf8"),
    );

    let mut status = paredit();
    status
        .args(["refactor", "status"])
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

    assert_status_preserves_file(&lisp_file, FUNCTION_FIXTURE, "hash mismatch fixture");
}

#[test]
fn cli_refactor_status_rejects_manifest_paths_outside_root_without_writing() {
    let dir = fresh_temp_dir("refactor status-root-guard");
    let root = dir.join("workspace");
    let outside_file = dir.join("outside.lisp");
    let manifest_file = dir.join("rename.preview.json");
    let original = "(defun old-name (x) x)\n";
    fs::create_dir_all(&root).expect("create guarded workspace");
    fs::write(&outside_file, original).expect("write outside fixture");
    write_manifest_fixture(
        &manifest_file,
        &String::from_utf8(preview_function_manifest(&outside_file, false)).expect("preview utf8"),
    );

    let mut status = paredit();
    status
        .args(["refactor", "status"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--root")
        .arg(&root)
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("outside refactor root"));

    assert_status_preserves_file(&outside_file, original, "unchanged outside fixture");
}
