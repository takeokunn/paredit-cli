use super::*;

mod apply;
mod check;
mod diff;
mod status;

#[test]
fn preview_manifest_out_writes_manifest_and_reports_matching_hash() {
    let dir = fresh_temp_dir("preview-manifest-out");
    let source = dir.join("source.lisp");
    let manifest = dir.join("preview.json");
    fs::write(
        &source,
        "(defun old-name (x) x)\n(defun caller () (old-name 1))\n",
    )
    .expect("write source fixture");

    let output = paredit()
        .args([
            "refactor", "preview", "--from", "old-name", "--to", "new-name",
        ])
        .arg("--manifest-out")
        .arg(&manifest)
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summary: serde_json::Value =
        serde_json::from_slice(&output).expect("summary is valid JSON");
    let reported_hash = summary["manifest_hash"].as_str().expect("manifest_hash");

    let manifest_text = fs::read_to_string(&manifest).expect("manifest was written");
    assert_eq!(reported_hash, stable_manifest_hash(&manifest_text));
    assert!(
        summary["next_actions"][1]
            .as_str()
            .expect("apply next action")
            .contains("--expect-manifest-hash")
    );

    paredit()
        .args(["refactor", "apply", "--expect-manifest-hash", reported_hash])
        .arg("--manifest")
        .arg(&manifest)
        .args(["--root", "/", "--write"])
        .assert()
        .success();
    let rewritten = fs::read_to_string(&source).expect("read rewritten source");
    assert!(rewritten.contains("(defun new-name (x) x)"), "{rewritten}");
}
