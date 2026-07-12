use super::*;

#[test]
fn cli_reports_cross_file_similarity_as_json() {
    let dir = fresh_temp_dir("similarity-json");
    let left = dir.join("left.lisp");
    let right = dir.join("right.lisp");
    fs::write(&left, "(defun alpha (x) (+ x 1))\n").unwrap();
    fs::write(&right, "(defun beta (y) (+ y 2))\n").unwrap();
    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold")
        .arg("0.8")
        .arg(&right)
        .arg(&left)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    let pairs = report["pairs"].as_array().unwrap();
    assert_eq!(report["pair_count"].as_u64().unwrap() as usize, pairs.len());
    assert!(!pairs.is_empty());
    assert!(
        pairs
            .iter()
            .all(|pair| pair["similarity"].as_f64().unwrap() >= 0.8)
    );
    assert_ne!(pairs[0]["left"]["path"], pairs[0]["right"]["path"]);
}

#[test]
fn cli_accepts_exact_similarity_threshold_endpoint() {
    let dir = fresh_temp_dir("similarity-threshold-one");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a b) (foo a b) (foo a c)\n").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    let pairs = report["pairs"].as_array().unwrap();
    assert!(!pairs.is_empty());
    assert!(
        pairs
            .iter()
            .all(|pair| pair["similarity"].as_f64() == Some(1.0))
    );
}

#[cfg(unix)]
#[test]
fn cli_deduplicates_the_same_canonical_input_path() {
    let dir = fresh_temp_dir("similarity-deduplicate-input");
    let file = dir.join("single.lisp");
    let alias = dir.join("alias.lisp");
    fs::write(&file, "(foo a b)\n").unwrap();
    std::os::unix::fs::symlink(&file, &alias).unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg(&file)
        .arg(&alias)
        .output()
        .unwrap();

    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["pair_count"].as_u64(), Some(0));
    assert_eq!(report["pairs"].as_array().unwrap().len(), 0);
}

#[test]
fn cli_renders_text_report() {
    let dir = fresh_temp_dir("similarity-text");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a b) (foo x y)\n").unwrap();
    paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold")
        .arg("0.8")
        .arg("--output")
        .arg("text")
        .arg(file)
        .assert()
        .success()
        .stdout(predicate::str::contains("pair_count\t1"))
        .stdout(predicate::str::contains("similarity="));
}

#[test]
fn cli_rejects_invalid_thresholds() {
    let dir = fresh_temp_dir("similarity-invalid");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a b)\n").unwrap();
    paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold")
        .arg("1.1")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--threshold must be between"));
    paredit()
        .args(["inspect", "similarity"])
        .arg("--min-node-count")
        .arg("1")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--min-node-count must be at least 2",
        ));
}

#[test]
fn fail_on_duplicates_preserves_stdout_report() {
    let dir = fresh_temp_dir("similarity-policy");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a b) (foo x y)\n").unwrap();
    paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold")
        .arg("0.8")
        .arg("--fail-on-duplicates")
        .arg(file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"pair_count\": 1"))
        .stderr(predicate::str::contains("similarity-report policy failed:"));
}

#[test]
fn cli_json_contract_reports_options_summary_and_pair_count() {
    let dir = fresh_temp_dir("similarity-json-contract");
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
        .arg(&file)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["options"]["threshold"], 0.0);
    assert_eq!(report["options"]["min_node_count"], 2);
    assert_eq!(report["options"]["min_line_span"], 1);
    assert_eq!(report["options"]["comparison_scope"], "all");
    assert_eq!(report["options"]["form_scope"], "all");
    assert_eq!(report["options"]["overlap_policy"], "all");
    assert_eq!(report["options"]["max_candidates"], Value::Null);
    assert_eq!(report["options"]["max_comparisons"], Value::Null);
    assert_eq!(report["options"]["max_results"], 1);
    assert_eq!(report["options"]["error_policy"], "fail");
    assert_eq!(report["summary"]["processed_files"], 1);
    assert_eq!(report["summary"]["skipped_error_files"], 0);
    assert_eq!(report["summary"]["matched_pairs"], 3);
    assert_eq!(report["summary"]["reported_pairs"], 1);
    assert_eq!(report["summary"]["truncated"], true);
    assert_eq!(report["summary"]["candidate_limit_reached"], false);
    assert_eq!(report["summary"]["omitted_candidates"], 0);
    assert_eq!(report["summary"]["comparison_limit_reached"], false);
    assert_eq!(report["summary"]["unprocessed_pairs"], 0);
    assert_eq!(report["pair_count"], 1);
    assert_eq!(report["pairs"].as_array().unwrap().len(), 1);
    assert_eq!(report["errors"].as_array().unwrap().len(), 0);
}
