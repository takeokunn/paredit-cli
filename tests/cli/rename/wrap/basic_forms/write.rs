use super::*;

#[test]
fn cli_writes_wrap_function_calls_and_skips_already_wrapped() {
    let dir = fresh_temp_dir("wrap-function-calls-write");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render ()\n  (fetch-user id)\n  (with-cache (fetch-user cached)))\n",
    )
    .expect("write lisp fixture");

    let output = paredit()
        .arg("wrap-function-calls")
        .arg(&lisp_file)
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
    let report = parse_wrap_report(&output.stdout).expect("parse wrap report");
    assert_eq!(report.call_count, 1);
    assert_eq!(report.skipped_already_wrapped_count, 1);
    assert_eq!(report.skipped_nested_count, 0);
    assert_eq!(report.files.first().map(|file| file.written), Some(true));

    let rewritten = fs::read_to_string(lisp_file).expect("read wrapped lisp");
    assert!(rewritten.contains("(with-cache (fetch-user id))"));
    assert!(rewritten.contains("(with-cache (fetch-user cached))"));
    assert!(!rewritten.contains("(with-cache (with-cache"));
}

#[test]
fn cli_wrap_function_calls_accepts_wrapper_template() {
    let dir = fresh_temp_dir("wrap-function-calls-template");
    let lisp_file = dir.join("service.lisp");
    let source = "(defun render ()\n  (fetch-user id))\n";
    fs::write(&lisp_file, source).expect("write lisp fixture");

    let output = paredit()
        .arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-tracing")
        .arg("--wrapper-template")
        .arg("(with-tracing :label 'fetch-user _)")
        .arg("--all-calls")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run paredit");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_wrap_report(&output.stdout).expect("parse wrap report");
    assert_eq!(
        report.wrapper_template.as_deref(),
        Some("(with-tracing :label 'fetch-user _)")
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("(with-tracing :label 'fetch-user (fetch-user id))")
    );

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read unchanged lisp"),
        source
    );
}

#[test]
fn cli_wrap_function_calls_can_target_call_path() {
    let dir = fresh_temp_dir("wrap-function-calls-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (fetch-user id) (fetch-user other))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
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
        "(defun render () (fetch-user id) (with-cache (fetch-user other)))\n"
    );
}

#[test]
fn cli_wrap_function_calls_skips_nested_all_call_rewrites() {
    let dir = fresh_temp_dir("wrap-function-calls-nested");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (fetch-user (fetch-user id)))\n",
    )
    .expect("write lisp fixture");

    let output = paredit()
        .arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--all-calls")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run paredit");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_wrap_report(&output.stdout).expect("parse wrap report");
    assert_eq!(report.call_count, 1);
    assert_eq!(report.skipped_nested_count, 1);
    assert_eq!(report.skipped_already_wrapped_count, 0);
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("(with-cache (fetch-user (fetch-user id)))")
    );
}
