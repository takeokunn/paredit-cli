use super::*;

#[test]
fn cli_e2e_applies_workspace_refactor_manifest_across_lisp_dialects() {
    let dir = fresh_temp_dir("workspace-refactor apply");
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
        .args(["refactor", "workspace-preview"])
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
        .args(["refactor", "apply"])
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

#[test]
fn cli_applies_workspace_refactor_manifest_with_hidden_and_generated_inputs() {
    let dir = fresh_temp_dir("workspace-refactor apply-hidden-generated");
    let src_dir = dir.join("src");
    let hidden_dir = dir.join(".hidden");
    let generated_dir = dir.join("target");
    let manifest_file = dir.join("workspace.preview.json");
    fs::create_dir_all(&src_dir).expect("create source dir");
    fs::create_dir_all(&hidden_dir).expect("create hidden dir");
    fs::create_dir_all(&generated_dir).expect("create generated dir");

    let src_file = src_dir.join("core.lisp");
    let hidden_file = hidden_dir.join("secret.lisp");
    let generated_file = generated_dir.join("generated.lisp");
    fs::write(
        &src_file,
        "(defun render-pane (pane) pane)\n(defun caller () (render-pane window))\n",
    )
    .expect("write source fixture");
    fs::write(
        &hidden_file,
        "(defun hidden-caller () (render-pane hidden-window))\n",
    )
    .expect("write hidden fixture");
    fs::write(
        &generated_file,
        "(defun generated-caller () (render-pane generated-window))\n",
    )
    .expect("write generated fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "workspace-preview"])
        .arg("--include-hidden")
        .arg("--include-generated")
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
        .arg("--require-changed-files")
        .arg("3")
        .arg("--require-definitions")
        .arg("1")
        .arg("--require-edits")
        .arg("4")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"discovered_file_count\": 3"))
        .stdout(predicate::str::contains("\"changed_file_count\": 3"))
        .get_output()
        .stdout
        .clone();
    fs::write(&manifest_file, preview_output).expect("write workspace manifest");

    let mut apply = paredit();
    apply
        .args(["refactor", "apply"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"applied\""))
        .stdout(predicate::str::contains("\"blocked_reasons\": []"))
        .stdout(predicate::str::contains("\"file_count\": 3"))
        .stdout(predicate::str::contains("\"changed_file_count\": 3"))
        .stdout(predicate::str::contains("\"written_file_count\": 3"))
        .stdout(predicate::str::contains("\"edit_count\": 4"))
        .stdout(predicate::str::contains("core.lisp"))
        .stdout(predicate::str::contains("secret.lisp"))
        .stdout(predicate::str::contains("generated.lisp"))
        .stdout(predicate::str::contains("\"applied\": true"));

    assert_eq!(
        fs::read_to_string(&src_file).expect("read rewritten source fixture"),
        "(defun paint-pane (pane) pane)\n(defun caller () (paint-pane window))\n"
    );
    assert_eq!(
        fs::read_to_string(&hidden_file).expect("read rewritten hidden fixture"),
        "(defun hidden-caller () (paint-pane hidden-window))\n"
    );
    assert_eq!(
        fs::read_to_string(&generated_file).expect("read rewritten generated fixture"),
        "(defun generated-caller () (paint-pane generated-window))\n"
    );
}

#[test]
fn cli_refuses_stale_workspace_refactor_apply_manifest_without_writing() {
    let dir = fresh_temp_dir("workspace-refactor apply-stale");
    let src_dir = dir.join("src");
    let manifest_file = dir.join("workspace.preview.json");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let src_file = src_dir.join("core.lisp");
    let caller_file = src_dir.join("caller.lisp");
    let original = "(defun render-pane (pane) pane)\n(defun caller () (render-pane window))\n";
    fs::write(&src_file, original).expect("write source fixture");
    fs::write(
        &caller_file,
        "(defun secondary-caller () (render-pane sidebar))\n",
    )
    .expect("write caller fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
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
        .get_output()
        .stdout
        .clone();
    fs::write(&manifest_file, preview_output).expect("write workspace manifest");

    let stale = "(defun render-pane (pane) (list pane))\n(defun caller () (render-pane window))\n";
    fs::write(&src_file, stale).expect("mutate source fixture after preview");

    let mut apply = paredit();
    apply
        .args(["refactor", "apply"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"status\": \"blocked\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"regenerate_refactor_preview\"",
        ))
        .stdout(predicate::str::contains("\"stale_files\""))
        .stdout(predicate::str::contains("\"applied\": false"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stdout(predicate::str::contains("\"stale_file_count\": 1"))
        .stdout(predicate::str::contains("\"input_hash_matches\": false"))
        .stderr(predicate::str::contains("refactor apply validation failed"));

    assert_eq!(
        fs::read_to_string(&src_file).expect("read stale source fixture"),
        stale
    );
    assert_eq!(
        fs::read_to_string(&caller_file).expect("read untouched caller fixture"),
        "(defun secondary-caller () (render-pane sidebar))\n"
    );
}

#[test]
fn cli_reports_policy_failed_workspace_refactor_apply_manifest_without_writing() {
    let dir = fresh_temp_dir("workspace-refactor apply-policy-failed");
    let src_dir = dir.join("src");
    let manifest_file = dir.join("workspace.preview.json");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let src_file = src_dir.join("core.lisp");
    let caller_file = src_dir.join("caller.lisp");
    let original = "(defun render-pane (pane) pane)\n(defun caller () (render-pane window))\n";
    fs::write(&src_file, original).expect("write source fixture");
    fs::write(
        &caller_file,
        "(defun secondary-caller () (render-pane sidebar))\n",
    )
    .expect("write caller fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
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
        .get_output()
        .stdout
        .clone();
    let manifest_text = String::from_utf8(preview_output)
        .expect("preview output is utf8")
        .replace("\"passed\": true", "\"passed\": false");
    fs::write(&manifest_file, manifest_text).expect("write policy-failed workspace manifest");

    let mut apply = paredit();
    apply
        .args(["refactor", "apply"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"status\": \"blocked\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"regenerate_refactor_preview\"",
        ))
        .stdout(predicate::str::contains(
            "\"manifest_policy_passed\": false",
        ))
        .stdout(predicate::str::contains("\"manifest_outputs_parse\": true"))
        .stdout(predicate::str::contains("\"manifest_policy_failed\""))
        .stdout(predicate::str::contains("\"applied\": false"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stderr(predicate::str::contains("manifest_policy_passed=false"));

    assert_eq!(
        fs::read_to_string(&src_file).expect("read policy failed source fixture"),
        original
    );
    assert_eq!(
        fs::read_to_string(&caller_file).expect("read policy failed caller fixture"),
        "(defun secondary-caller () (render-pane sidebar))\n"
    );
}

#[test]
fn cli_reports_unparseable_workspace_refactor_apply_manifest_outputs_without_writing() {
    let dir = fresh_temp_dir("workspace-refactor apply-unparseable-outputs");
    let src_dir = dir.join("src");
    let manifest_file = dir.join("workspace.preview.json");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let src_file = src_dir.join("core.lisp");
    let caller_file = src_dir.join("caller.lisp");
    let original = "(defun render-pane (pane) pane)\n(defun caller () (render-pane window))\n";
    fs::write(&src_file, original).expect("write source fixture");
    fs::write(
        &caller_file,
        "(defun secondary-caller () (render-pane sidebar))\n",
    )
    .expect("write caller fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
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
        .get_output()
        .stdout
        .clone();
    let manifest_text = String::from_utf8(preview_output)
        .expect("preview output is utf8")
        .replace(
            "\"all_outputs_parse\": true",
            "\"all_outputs_parse\": false",
        );
    fs::write(&manifest_file, manifest_text).expect("write unparseable-output workspace manifest");

    let mut apply = paredit();
    apply
        .args(["refactor", "apply"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .failure()
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
        .stdout(predicate::str::contains("\"applied\": false"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stderr(predicate::str::contains("manifest_outputs_parse=false"));

    assert_eq!(
        fs::read_to_string(&src_file).expect("read unparseable source fixture"),
        original
    );
    assert_eq!(
        fs::read_to_string(&caller_file).expect("read unparseable caller fixture"),
        "(defun secondary-caller () (render-pane sidebar))\n"
    );
}

#[test]
fn cli_reports_output_hash_mismatch_workspace_refactor_apply_manifest_without_writing() {
    let dir = fresh_temp_dir("workspace-refactor apply-output-hash-mismatch");
    let src_dir = dir.join("src");
    let manifest_file = dir.join("workspace.preview.json");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let src_file = src_dir.join("core.lisp");
    let caller_file = src_dir.join("caller.lisp");
    let original = "(defun render-pane (pane) pane)\n(defun caller () (render-pane window))\n";
    fs::write(&src_file, original).expect("write source fixture");
    fs::write(
        &caller_file,
        "(defun secondary-caller () (render-pane sidebar))\n",
    )
    .expect("write caller fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
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
        .get_output()
        .stdout
        .clone();
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&preview_output).expect("preview output is json");
    manifest["files"][0]["output_hash"] = serde_json::Value::String("fnv1a64:deadbeef".into());
    fs::write(
        &manifest_file,
        serde_json::to_string_pretty(&manifest).expect("serialize output hash mismatch manifest"),
    )
    .expect("write output hash mismatch manifest");

    let mut apply = paredit();
    apply
        .args(["refactor", "apply"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--write")
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
        .stdout(predicate::str::contains("\"applied\": false"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stderr(predicate::str::contains("output_hash_mismatches=1"));

    assert_eq!(
        fs::read_to_string(&src_file).expect("read output hash mismatch source fixture"),
        original
    );
    assert_eq!(
        fs::read_to_string(&caller_file).expect("read output hash mismatch caller fixture"),
        "(defun secondary-caller () (render-pane sidebar))\n"
    );
}

#[test]
fn cli_reports_manifest_flag_mismatch_workspace_refactor_apply_manifest_without_writing() {
    let dir = fresh_temp_dir("workspace-refactor apply-flag-mismatch");
    let src_dir = dir.join("src");
    let manifest_file = dir.join("workspace.preview.json");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let src_file = src_dir.join("core.lisp");
    let caller_file = src_dir.join("caller.lisp");
    let original = "(defun render-pane (pane) pane)\n(defun caller () (render-pane window))\n";
    fs::write(&src_file, original).expect("write source fixture");
    fs::write(
        &caller_file,
        "(defun secondary-caller () (render-pane sidebar))\n",
    )
    .expect("write caller fixture");

    let mut preview = paredit();
    let preview_output = preview
        .args(["refactor", "workspace-preview"])
        .arg("--from")
        .arg("render-pane")
        .arg("--to")
        .arg("paint-pane")
        .arg("--mode")
        .arg("function")
        .arg("--fail-on-no-change")
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
        .get_output()
        .stdout
        .clone();
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&preview_output).expect("preview output is json");
    manifest["files"][0]["changed"] = serde_json::Value::Bool(false);
    fs::write(
        &manifest_file,
        serde_json::to_string_pretty(&manifest).expect("serialize manifest flag mismatch manifest"),
    )
    .expect("write manifest flag mismatch manifest");

    let mut apply = paredit();
    apply
        .args(["refactor", "apply"])
        .arg("--manifest")
        .arg(&manifest_file)
        .arg("--write")
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
        .stdout(predicate::str::contains("\"applied\": false"))
        .stdout(predicate::str::contains("\"written_file_count\": 0"))
        .stderr(predicate::str::contains("manifest_flag_mismatches=1"));

    assert_eq!(
        fs::read_to_string(&src_file).expect("read manifest flag mismatch source fixture"),
        original
    );
    assert_eq!(
        fs::read_to_string(&caller_file).expect("read manifest flag mismatch caller fixture"),
        "(defun secondary-caller () (render-pane sidebar))\n"
    );
}
