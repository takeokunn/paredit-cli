use super::*;

#[test]
fn cli_requires_file_for_replace_forms_writes() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "replace-forms",
        "--path",
        "0",
        "--with",
        "(x)",
        "--write",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "replace-forms --write requires --file",
    ));
}

#[test]
fn cli_plans_replace_forms_for_duplicate_refactor() {
    let dir = fresh_temp_dir("replace-forms-plan");
    let lisp_file = dir.join("suite.lisp");
    let source = "(deftest split-a () (is (= 1 (pane-count))))\n(deftest split-b () (is (= 2 (pane-count))))\n(defun keep () :ok)\n";
    fs::write(&lisp_file, source).expect("write duplicate fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replace-forms")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0")
        .arg("--path")
        .arg("1")
        .arg("--with")
        .arg("(run-split-case)")
        .arg("--require-same-shape")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"path_count\": 2"))
        .stdout(predicate::str::contains(
            "\"replacement\": \"(run-split-case)\"",
        ))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"text\": \"(deftest split-a"))
        .stdout(predicate::str::contains(
            "(run-split-case)\\n(run-split-case)",
        ));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read planned duplicate fixture"),
        source
    );
}

#[test]
fn cli_writes_replace_forms_in_reverse_span_order() {
    let dir = fresh_temp_dir("replace-forms-write");
    let lisp_file = dir.join("suite.lisp");
    fs::write(
        &lisp_file,
        "(deftest split-a () (is (= 1 (pane-count))))\n(deftest split-b () (is (= 2 (pane-count))))\n(defun keep () :ok)\n",
    )
    .expect("write duplicate fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replace-forms")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0")
        .arg("--path")
        .arg("1")
        .arg("--with")
        .arg("(run-split-case)")
        .arg("--require-same-shape")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten duplicate fixture"),
        "(run-split-case)\n(run-split-case)\n(defun keep () :ok)\n"
    );
}

#[test]
fn cli_rejects_replace_forms_overlapping_paths() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "replace-forms",
        "--path",
        "0",
        "--path",
        "0.1",
        "--with",
        "(x)",
    ])
    .write_stdin("(outer (inner 1))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("paths must not overlap"));
}

#[test]
fn cli_rejects_replace_forms_shape_mismatch_when_required() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "replace-forms",
        "--path",
        "0",
        "--path",
        "1",
        "--with",
        "(x)",
        "--require-same-shape",
    ])
    .write_stdin("(a 1) (b 1 2)")
    .assert()
    .failure()
    .stderr(predicate::str::contains("share shape"));
}
