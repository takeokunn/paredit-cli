use super::*;

#[test]
fn cli_writes_unwrap_function_calls_and_skips_non_unary_wrappers() {
    let dir = fresh_temp_dir("unwrap-function-calls-write");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render ()\n  (with-cache (fetch-user id))\n  (with-cache (fetch-user cached) :ttl 60))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--all-calls")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains(
            "\"skippedNonUnaryWrapperCount\": 1",
        ))
        .stdout(predicate::str::contains("\"written\": true"));

    let rewritten = fs::read_to_string(lisp_file).expect("read unwrapped lisp");
    assert!(rewritten.contains("(fetch-user id)"));
    assert!(rewritten.contains("(with-cache (fetch-user cached) :ttl 60)"));
    assert!(!rewritten.contains("(with-cache (fetch-user id))"));
}

#[test]
fn cli_unwrap_function_calls_can_target_call_path() {
    let dir = fresh_temp_dir("unwrap-function-calls-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (with-cache (fetch-user id)) (with-cache (fetch-user other)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--call-path")
        .arg("0.4")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"path\": \"0.4\""));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read path-targeted lisp"),
        "(defun render () (with-cache (fetch-user id)) (fetch-user other))\n"
    );
}

#[test]
fn cli_unwrap_function_calls_aggregates_counts_across_multiple_files() {
    let dir = fresh_temp_dir("unwrap-function-calls-multi-file");
    let changed_file = dir.join("changed.lisp");
    let unchanged_file = dir.join("unchanged.lisp");
    fs::write(
        &changed_file,
        "(defun render () (with-cache (fetch-user id)))\n",
    )
    .expect("write changed fixture");
    fs::write(
        &unchanged_file,
        "(defun render () (with-cache (fetch-user cached) :ttl 60))\n",
    )
    .expect("write unchanged fixture");

    let output = paredit()
        .arg("unwrap-function-calls")
        .arg(&changed_file)
        .arg(&unchanged_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--all-calls")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run paredit");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = parse_unwrap_report(&output.stdout).expect("parse unwrap report");
    assert_eq!(report.call_count, 1);
    assert_eq!(report.skipped_non_unary_wrapper_count, 1);
    assert_eq!(
        report.files,
        vec![
            CliWrittenFileReport { written: true },
            CliWrittenFileReport { written: false },
        ]
    );

    assert_eq!(
        fs::read_to_string(changed_file).expect("read changed file"),
        "(defun render () (fetch-user id))\n"
    );
    assert_eq!(
        fs::read_to_string(unchanged_file).expect("read unchanged file"),
        "(defun render () (with-cache (fetch-user cached) :ttl 60))\n"
    );
}

#[test]
fn cli_unwrap_function_calls_skips_nested_all_call_rewrites() {
    let dir = fresh_temp_dir("unwrap-function-calls-nested");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (with-cache (fetch-user (with-cache (fetch-user id)))))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--all-calls")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"skippedNestedCount\": 1"))
        .stdout(predicate::str::contains(
            "(fetch-user (with-cache (fetch-user id)))",
        ));
}
