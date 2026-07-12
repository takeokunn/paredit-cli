use super::*;

#[test]
fn max_results_does_not_hide_duplicates_from_failure_policy() {
    let dir = fresh_temp_dir("similarity-truncated-policy");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (foo b) (foo c)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold")
        .arg("0")
        .arg("--min-node-count")
        .arg("2")
        .arg("--overlap-policy")
        .arg("all")
        .arg("--max-results")
        .arg("1")
        .arg("--fail-on-duplicates")
        .arg(&file)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["summary"]["matched_pairs"], 3);
    assert_eq!(report["pair_count"], 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("3 duplicate pair(s) found"));
}

#[test]
fn max_candidates_reports_precise_omissions_and_fails_closed() {
    let dir = fresh_temp_dir("similarity-candidate-limit");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (bar b) (baz c)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg("--min-node-count=2")
        .arg("--form-scope=top-level")
        .arg("--max-candidates=2")
        .arg("--fail-on-duplicates")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["options"]["max_candidates"], 2);
    assert_eq!(report["summary"]["candidate_limit_reached"], true);
    assert_eq!(report["summary"]["omitted_candidates"], 1);
    assert_eq!(report["summary"]["matched_pairs"], 0);
    assert!(String::from_utf8_lossy(&output.stderr).contains(
        "similarity-report policy indeterminate: candidate limit reached with 1 candidate(s) omitted"
    ));
}

#[test]
fn max_candidates_not_reached_reports_complete_collection() {
    let dir = fresh_temp_dir("similarity-candidate-limit-complete");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (bar b) (baz c)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg("--min-node-count=2")
        .arg("--form-scope=top-level")
        .arg("--max-candidates=3")
        .arg("--fail-on-duplicates")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["summary"]["candidate_limit_reached"], false);
    assert_eq!(report["summary"]["omitted_candidates"], 0);
}

#[test]
fn max_comparisons_is_reported_and_failure_uses_processed_matches() {
    let dir = fresh_temp_dir("similarity-comparison-limit");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (foo b) (foo c)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=0")
        .arg("--min-node-count=2")
        .arg("--overlap-policy=all")
        .arg("--max-comparisons=1")
        .arg("--fail-on-duplicates")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["options"]["max_comparisons"], 1);
    assert_eq!(report["summary"]["possible_pairs"], 3);
    assert_eq!(report["summary"]["evaluated_pairs"], 1);
    assert_eq!(report["summary"]["unprocessed_pairs"], 2);
    assert_eq!(report["summary"]["comparison_limit_reached"], true);
    assert_eq!(report["summary"]["matched_pairs"], 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("1 duplicate pair(s) found"));
}

#[test]
fn fail_on_duplicates_rejects_an_indeterminate_comparison_limit() {
    let dir = fresh_temp_dir("similarity-comparison-limit-indeterminate");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (bar b) (foo a)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg("--min-node-count=2")
        .arg("--form-scope=top-level")
        .arg("--overlap-policy=all")
        .arg("--max-comparisons=1")
        .arg("--fail-on-duplicates")
        .arg(&file)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["summary"]["matched_pairs"], 0);
    assert_eq!(report["summary"]["comparison_limit_reached"], true);
    assert_eq!(report["summary"]["unprocessed_pairs"], 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains(
        "similarity-report policy indeterminate: comparison limit reached with 2 pair(s) unprocessed"
    ));
}

#[test]
fn fail_on_duplicates_succeeds_when_comparison_limit_is_not_reached() {
    let dir = fresh_temp_dir("similarity-comparison-limit-complete");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (bar b) (baz c)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg("--min-node-count=2")
        .arg("--form-scope=top-level")
        .arg("--overlap-policy=all")
        .arg("--max-comparisons=3")
        .arg("--fail-on-duplicates")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["summary"]["matched_pairs"], 0);
    assert_eq!(report["summary"]["comparison_limit_reached"], false);
    assert_eq!(report["summary"]["unprocessed_pairs"], 0);
}

#[test]
fn comparison_scope_accepts_file_alias() {
    let dir = fresh_temp_dir("similarity-comparison-scope-alias");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (bar b)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=0")
        .arg("--min-node-count=2")
        .arg("--comparison-scope=file")
        .arg("--overlap-policy=all")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["options"]["comparison_scope"], "same-file");
    assert_eq!(report["summary"]["possible_pairs"], 1);
    assert_eq!(report["summary"]["evaluated_pairs"], 1);
}

#[test]
fn text_output_reports_max_comparisons_summary() {
    let dir = fresh_temp_dir("similarity-comparison-limit-text");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (foo b) (foo c)\n").unwrap();

    paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=0")
        .arg("--min-node-count=2")
        .arg("--overlap-policy=all")
        .arg("--max-comparisons=1")
        .arg("--output=text")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("max_comparisons\t1"))
        .stdout(predicate::str::contains("comparison_limit_reached\ttrue"))
        .stdout(predicate::str::contains("unprocessed_pairs\t2"));
}

#[test]
fn cli_rejects_non_finite_thresholds_and_zero_max_results() {
    let dir = fresh_temp_dir("similarity-invalid-numeric-options");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a b)\n").unwrap();

    for threshold in ["NaN", "inf", "-inf"] {
        paredit()
            .args(["inspect", "similarity"])
            .arg(format!("--threshold={threshold}"))
            .arg(&file)
            .assert()
            .failure()
            .stderr(predicate::str::contains("--threshold must be between"));
    }
    paredit()
        .args(["inspect", "similarity"])
        .arg("--min-line-span=0")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--min-line-span must be at least 1",
        ));
    paredit()
        .args(["inspect", "similarity"])
        .arg("--max-results")
        .arg("0")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--max-results must be at least 1"));
    paredit()
        .args(["inspect", "similarity"])
        .arg("--max-comparisons=0")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--max-comparisons must be at least 1",
        ));
    paredit()
        .args(["inspect", "similarity"])
        .arg("--max-candidates=0")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--max-candidates must be at least 1",
        ));
}
