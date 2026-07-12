use super::*;

#[test]
fn cli_fails_on_malformed_input_by_default() {
    let dir = fresh_temp_dir("similarity-error-fail");
    let invalid = dir.join("invalid.lisp");
    fs::write(&invalid, "(").unwrap();

    paredit()
        .args(["inspect", "similarity"])
        .arg(&invalid)
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse"))
        .stderr(predicate::str::contains(invalid.display().to_string()));
}

#[test]
fn cli_skips_malformed_input_and_reports_stable_json_errors() {
    let dir = fresh_temp_dir("similarity-error-skip");
    let valid = dir.join("valid.lisp");
    let invalid = dir.join("invalid.lisp");
    fs::write(&valid, "(foo a b) (foo a b)\n").unwrap();
    fs::write(&invalid, "(").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg("--error-policy=skip")
        .arg(&valid)
        .arg(&invalid)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["options"]["error_policy"], "skip");
    assert_eq!(report["summary"]["scanned_files"], 2);
    assert_eq!(report["summary"]["processed_files"], 1);
    assert_eq!(report["summary"]["skipped_error_files"], 1);
    assert_eq!(report["pair_count"], 1);
    let errors = report["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["path"], invalid.display().to_string());
    assert_eq!(errors[0]["stage"], "parse");
    assert!(
        !errors[0]["message"]
            .as_str()
            .unwrap()
            .contains(&invalid.display().to_string())
    );
}

#[test]
fn cli_skip_succeeds_with_an_empty_report_when_all_inputs_are_invalid() {
    let dir = fresh_temp_dir("similarity-error-all-invalid");
    let first = dir.join("a.lisp");
    let second = dir.join("b.lisp");
    fs::write(&first, "(").unwrap();
    fs::write(&second, "(").unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--error-policy=skip")
        .arg(&second)
        .arg(&first)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["summary"]["processed_files"], 0);
    assert_eq!(report["summary"]["skipped_error_files"], 2);
    assert_eq!(report["pair_count"], 0);
    let errors = report["errors"].as_array().unwrap();
    assert_eq!(errors[0]["path"], first.display().to_string());
    assert_eq!(errors[1]["path"], second.display().to_string());
}

#[test]
fn cli_skip_still_applies_fail_on_duplicates_to_successful_files() {
    let dir = fresh_temp_dir("similarity-error-policy-duplicates");
    let valid = dir.join("valid.lisp");
    let invalid = dir.join("invalid.lisp");
    fs::write(&valid, "(foo a b) (foo a b)\n").unwrap();
    fs::write(&invalid, "(").unwrap();

    paredit()
        .args(["inspect", "similarity"])
        .arg("--threshold=1")
        .arg("--error-policy=skip")
        .arg("--fail-on-duplicates")
        .arg(&invalid)
        .arg(&valid)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"skipped_error_files\": 1"))
        .stdout(predicate::str::contains("\"pair_count\": 1"))
        .stderr(predicate::str::contains("similarity-report policy failed:"));
}

#[test]
fn fail_on_duplicates_rejects_skipped_files_as_indeterminate() {
    let dir = fresh_temp_dir("similarity-error-policy-indeterminate");
    let valid = dir.join("valid.lisp");
    let invalid = dir.join("invalid.lisp");
    fs::write(&valid, "(foo a b)\n").unwrap();
    fs::write(&invalid, [0xff, 0xfe]).unwrap();

    let output = paredit()
        .args(["inspect", "similarity"])
        .arg("--error-policy=skip")
        .arg("--fail-on-duplicates")
        .arg(&valid)
        .arg(&invalid)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let report = parse_similarity_report(&output.stdout);
    assert_eq!(report["pair_count"], 0);
    assert_eq!(report["summary"]["skipped_error_files"], 1);
    let errors = report["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["path"], invalid.display().to_string());
    assert_eq!(errors[0]["stage"], "read");
    assert!(String::from_utf8_lossy(&output.stderr).contains(
        "similarity-report policy indeterminate: 1 file(s) skipped due to processing errors"
    ));
}

#[test]
fn cli_text_report_includes_skipped_file_summary_and_error() {
    let dir = fresh_temp_dir("similarity-error-text");
    let invalid = dir.join("invalid.lisp");
    fs::write(&invalid, "(").unwrap();

    paredit()
        .args(["inspect", "similarity"])
        .arg("--error-policy=skip")
        .arg("--output=text")
        .arg(&invalid)
        .assert()
        .success()
        .stdout(predicate::str::contains("processed_files\t0"))
        .stdout(predicate::str::contains("skipped_error_files\t1"))
        .stdout(predicate::str::contains(format!(
            "error\t{}\tparse\t",
            invalid.display()
        )));
}
