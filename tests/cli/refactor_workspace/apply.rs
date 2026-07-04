use super::*;

#[test]
fn cli_e2e_applies_workspace_refactor_manifest_across_lisp_dialects() {
    let dir = fresh_temp_dir("workspace-refactor-apply");
    let src_dir = dir.join("src");
    let manifest_file = dir.join("workspace.preview.json");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let lisp_file = src_dir.join("core.lisp");
    let elisp_file = src_dir.join("ui.el");
    let ignored = src_dir.join("notes.txt");
    fs::write(
        &lisp_file,
        "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n",
    )
    .expect("write common lisp fixture");
    fs::write(
        &elisp_file,
        "(defun ui () (old-name 2))\n(message \"old-name\")\n",
    )
    .expect("write emacs lisp fixture");
    fs::write(&ignored, "old-name is plain text").expect("write ignored fixture");

    let mut preview = paredit();
    let preview_output = preview
        .arg("workspace-refactor-preview")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
        .arg("--fail-on-parse-error")
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
        .stdout(predicate::str::contains("\"discovered_file_count\": 2"))
        .stdout(predicate::str::contains("\"unknown\": 1"))
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .get_output()
        .stdout
        .clone();
    fs::write(&manifest_file, preview_output).expect("write workspace manifest");

    let mut apply = paredit();
    apply
        .arg("refactor-apply")
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"mode\": \"function\""))
        .stdout(predicate::str::contains("\"status\": \"applied\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"run_verification_or_review_diff\"",
        ))
        .stdout(predicate::str::contains("\"blocked_reasons\": []"))
        .stdout(predicate::str::contains("\"steps\": ["))
        .stdout(predicate::str::contains("\"name\": \"apply-write\""))
        .stdout(predicate::str::contains("\"status\": \"passed\""))
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"changed_file_count\": 2"))
        .stdout(predicate::str::contains("\"changed_files\": ["))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("ui.el"))
        .stdout(predicate::str::contains("\"written_file_count\": 2"))
        .stdout(predicate::str::contains("\"edit_count\": 3"))
        .stdout(predicate::str::contains("\"applied\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten common lisp fixture"),
        "(defun new-name (x) x)\n(defun caller () (new-name 1) old-name)\n"
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
