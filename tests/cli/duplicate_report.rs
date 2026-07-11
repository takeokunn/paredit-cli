use super::*;

#[test]
fn cli_reports_duplicate_structural_forms_for_refactor_planning() {
    let dir = fresh_temp_dir("duplicate-report");
    let test_file = dir.join("suite.lisp");
    fs::write(
        &test_file,
        "(deftest split-a () (is (= 1 (pane-count))))\n\
         (deftest split-b () (is (= 2 (pane-count))))\n\
         (defun helper (x) (+ x 1))\n",
    )
    .expect("write duplicate fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("duplicates")
        .arg("--min-node-count")
        .arg("8")
        .arg("--output")
        .arg("json")
        .arg(&test_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"group_count\": 1"))
        .stdout(predicate::str::contains("\"count\": 2"))
        .stdout(predicate::str::contains("\"head\": \"deftest\""))
        .stdout(predicate::str::contains("\"form_path\": \"0\""))
        .stdout(predicate::str::contains("\"form_path\": \"1\""))
        .stdout(predicate::str::contains("split-a"))
        .stdout(predicate::str::contains("split-b"));
}

#[test]
fn cli_plans_replacement_batches_from_duplicate_forms() {
    let dir = fresh_temp_dir("replacement-plan");
    let primary_file = dir.join("suite.lisp");
    let secondary_file = dir.join("other.lisp");
    fs::write(
        &primary_file,
        "(deftest split-a () (is (= 1 (pane-count))))\n\
         (deftest split-b () (is (= 2 (pane-count))))\n\
         (defun keep () :ok)\n",
    )
    .expect("write primary duplicate fixture");
    fs::write(
        &secondary_file,
        "(deftest split-c () (is (= 3 (pane-count))))\n",
    )
    .expect("write secondary duplicate fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replacement-plan")
        .arg("--min-node-count")
        .arg("8")
        .arg("--replacement")
        .arg("(run-split-case)")
        .arg("--output")
        .arg("json")
        .arg(&primary_file)
        .arg(&secondary_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"batch_count\": 1"))
        .stdout(predicate::str::contains("\"form_count\": 2"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains(
            "\"replacement\": \"(run-split-case)\"",
        ))
        .stdout(predicate::str::contains("\"replace_forms_args\""))
        .stdout(predicate::str::contains("\"paredit\""))
        .stdout(predicate::str::contains("\"replace-forms\""))
        .stdout(predicate::str::contains("\"--require-same-shape\""))
        .stdout(predicate::str::contains("\"0\""))
        .stdout(predicate::str::contains("\"1\""))
        .stdout(predicate::str::contains("split-a"))
        .stdout(predicate::str::contains("split-b"))
        .stdout(predicate::str::contains(
            format!("--file {}", primary_file.display()).as_str(),
        ));
}

#[test]
fn cli_plans_replacement_batches_can_keep_canonical_first_form() {
    let dir = fresh_temp_dir("replacement-plan-keep-first");
    let lisp_file = dir.join("suite.lisp");
    fs::write(
        &lisp_file,
        "(deftest split-a () (is (= 1 (pane-count))))\n\
         (deftest split-b () (is (= 2 (pane-count))))\n\
         (deftest split-c () (is (= 3 (pane-count))))\n",
    )
    .expect("write duplicate fixture");

    let mut cmd = paredit();
    let output = cmd
        .arg("refactor")
        .arg("replacement-plan")
        .arg("--min-node-count")
        .arg("8")
        .arg("--keep-first")
        .arg("--replacement")
        .arg("(run-split-case)")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&output).expect("replacement-plan json");
    let batch = &report["batches"][0];

    assert_eq!(report["batch_count"], 1);
    assert_eq!(report["candidate_form_count"], 3);
    assert_eq!(report["form_count"], 2);
    assert_eq!(batch["candidate_count"], 3);
    assert_eq!(batch["replacement_count"], 2);
    assert_eq!(batch["keep_first"], true);
    assert_eq!(batch["kept_form"]["form_path"], "0");
    assert_eq!(batch["paths"], serde_json::json!(["1", "2"]));
    assert!(
        batch["command"]
            .as_str()
            .expect("command")
            .contains("--path 1")
    );
    assert!(
        batch["command"]
            .as_str()
            .expect("command")
            .contains("--path 2")
    );
    assert!(
        !batch["command"]
            .as_str()
            .expect("command")
            .contains("--path 0")
    );
}

#[test]
fn cli_filters_replacement_plan_by_per_file_group_size() {
    let dir = fresh_temp_dir("replacement-plan-min-size");
    let lisp_file = dir.join("suite.lisp");
    fs::write(
        &lisp_file,
        "(deftest split-a () (is (= 1 (pane-count))))\n\
         (deftest split-b () (is (= 2 (pane-count))))\n",
    )
    .expect("write duplicate fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replacement-plan")
        .arg("--min-node-count")
        .arg("8")
        .arg("--min-group-size")
        .arg("3")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"batch_count\": 0"))
        .stdout(predicate::str::contains("\"form_count\": 0"));
}

#[test]
fn cli_uses_review_placeholder_for_default_replacement_plan_output() {
    let dir = fresh_temp_dir("replacement-plan-default-placeholder");
    let lisp_file = dir.join("suite.lisp");
    fs::write(
        &lisp_file,
        "(deftest split-a () (is (= 1 (pane-count))))\n\
         (deftest split-b () (is (= 2 (pane-count))))\n",
    )
    .expect("write duplicate fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replacement-plan")
        .arg("--min-node-count")
        .arg("8")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"replacement\": \"(__review_replacement__)\"",
        ))
        .stdout(predicate::str::contains("(__review_replacement__)"));
}
