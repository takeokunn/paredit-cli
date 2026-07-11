use std::fs;

use predicates::prelude::*;
use serde_json::Value;

use super::{fresh_temp_dir, paredit};

#[test]
fn cli_reports_cross_file_similarity_as_json() {
    let dir = fresh_temp_dir("similarity-json");
    let left = dir.join("left.lisp");
    let right = dir.join("right.lisp");
    fs::write(&left, "(defun alpha (x) (+ x 1))\n").unwrap();
    fs::write(&right, "(defun beta (y) (+ y 2))\n").unwrap();
    let output = paredit()
        .arg("similarity-report")
        .arg("--threshold")
        .arg("0.8")
        .arg(&right)
        .arg(&left)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let pairs = report["pairs"].as_array().unwrap();
    assert_eq!(report["pair_count"].as_u64().unwrap() as usize, pairs.len());
    assert!(!pairs.is_empty());
    assert!(pairs
        .iter()
        .all(|pair| pair["similarity"].as_f64().unwrap() >= 0.8));
    assert_ne!(pairs[0]["left"]["path"], pairs[0]["right"]["path"]);
}

#[test]
fn cli_accepts_exact_similarity_threshold_endpoint() {
    let dir = fresh_temp_dir("similarity-threshold-one");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a b) (foo a b) (foo a c)\n").unwrap();

    let output = paredit()
        .arg("similarity-report")
        .arg("--threshold=1")
        .arg(&file)
        .output()
        .unwrap();

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let pairs = report["pairs"].as_array().unwrap();
    assert!(!pairs.is_empty());
    assert!(pairs
        .iter()
        .all(|pair| pair["similarity"].as_f64() == Some(1.0)));
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
        .arg("similarity-report")
        .arg(&file)
        .arg(&alias)
        .output()
        .unwrap();

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["pair_count"].as_u64(), Some(0));
    assert_eq!(report["pairs"].as_array().unwrap().len(), 0);
}

#[test]
fn cli_renders_text_report() {
    let dir = fresh_temp_dir("similarity-text");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a b) (foo x y)\n").unwrap();
    paredit()
        .arg("similarity-report")
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
        .arg("similarity-report")
        .arg("--threshold")
        .arg("1.1")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--threshold must be between"));
    paredit()
        .arg("similarity-report")
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
        .arg("similarity-report")
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
fn cli_recursively_discovers_sources_and_skips_hidden_and_generated_directories() {
    let dir = fresh_temp_dir("similarity-discovery");
    let nested = dir.join("src").join("nested");
    let hidden = dir.join(".private");
    let generated = dir.join("target");
    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(&hidden).unwrap();
    fs::create_dir_all(&generated).unwrap();
    fs::write(nested.join("left.lisp"), "(foo a b)\n").unwrap();
    fs::write(dir.join("right.lisp"), "(foo x y)\n").unwrap();
    fs::write(hidden.join("hidden.lisp"), "(foo h i)\n").unwrap();
    fs::write(generated.join("generated.lisp"), "(foo g h)\n").unwrap();

    let output = paredit()
        .arg("similarity-report")
        .arg("--threshold")
        .arg("0")
        .arg("--min-node-count")
        .arg("2")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["summary"]["scanned_files"], 2);
    assert_eq!(report["summary"]["skipped_hidden"], 1);
    assert_eq!(report["summary"]["skipped_generated"], 1);
    assert_eq!(report["summary"]["matched_pairs"], 1);
}

#[test]
fn cli_dialect_override_includes_unknown_extensions() {
    let dir = fresh_temp_dir("similarity-dialect-override");
    let left = dir.join("left.txt");
    let right = dir.join("right.data");
    fs::write(&left, "(foo a b)\n").unwrap();
    fs::write(&right, "(foo x y)\n").unwrap();

    let output = paredit()
        .arg("similarity-report")
        .arg("--dialect")
        .arg("common-lisp")
        .arg("--threshold")
        .arg("0")
        .arg("--min-node-count")
        .arg("2")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["options"]["dialect"], "common-lisp");
    assert_eq!(report["summary"]["scanned_files"], 2);
    assert_eq!(report["summary"]["skipped_unknown"], 0);
    assert!(report["pairs"].as_array().unwrap().iter().all(|pair| {
        pair["left"]["dialect"] == "common-lisp" && pair["right"]["dialect"] == "common-lisp"
    }));
}

#[test]
fn cli_json_contract_reports_options_summary_and_pair_count() {
    let dir = fresh_temp_dir("similarity-json-contract");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (foo b) (foo c)\n").unwrap();

    let output = paredit()
        .arg("similarity-report")
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
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["options"]["threshold"], 0.0);
    assert_eq!(report["options"]["min_node_count"], 2);
    assert_eq!(report["options"]["overlap_policy"], "all");
    assert_eq!(report["options"]["max_results"], 1);
    assert_eq!(report["summary"]["matched_pairs"], 3);
    assert_eq!(report["summary"]["reported_pairs"], 1);
    assert_eq!(report["summary"]["truncated"], true);
    assert_eq!(report["pair_count"], 1);
    assert_eq!(report["pairs"].as_array().unwrap().len(), 1);
}

#[test]
fn max_results_does_not_hide_duplicates_from_failure_policy() {
    let dir = fresh_temp_dir("similarity-truncated-policy");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a) (foo b) (foo c)\n").unwrap();

    let output = paredit()
        .arg("similarity-report")
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
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["summary"]["matched_pairs"], 3);
    assert_eq!(report["pair_count"], 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("3 duplicate pair(s) found"));
}

#[test]
fn cli_rejects_non_finite_thresholds_and_zero_max_results() {
    let dir = fresh_temp_dir("similarity-invalid-numeric-options");
    let file = dir.join("suite.lisp");
    fs::write(&file, "(foo a b)\n").unwrap();

    for threshold in ["NaN", "inf", "-inf"] {
        paredit()
            .arg("similarity-report")
            .arg(format!("--threshold={threshold}"))
            .arg(&file)
            .assert()
            .failure()
            .stderr(predicate::str::contains("--threshold must be between"));
    }
    paredit()
        .arg("similarity-report")
        .arg("--max-results")
        .arg("0")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--max-results must be at least 1"));
}
