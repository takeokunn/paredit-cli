use super::*;

#[test]
fn cli_plans_unwrap_call_without_writing() {
    let mut cmd = paredit();
    cmd.args([
        "unwrap-call",
        "--path",
        "0",
        "--function",
        "with-cache",
        "--argument-index",
        "0",
        "--output",
        "json",
    ])
    .write_stdin("(with-cache (fetch-user id) :ttl 60)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"function\": \"with-cache\""))
    .stdout(predicate::str::contains("\"argumentIndex\": 0"))
    .stdout(predicate::str::contains("\"callArgumentCount\": 3"))
    .stdout(predicate::str::contains(
        "\"replacement\": \"(fetch-user id)\"",
    ))
    .stdout(predicate::str::contains("\"written\": false"));
}

#[test]
fn cli_writes_unwrap_call_for_emacs_lisp_file() {
    let dir = fresh_temp_dir("unwrap-call-write");
    let el_file = dir.join("wrapper.el");
    fs::write(&el_file, "(with-current-buffer buf (message \"ready\"))\n")
        .expect("write emacs lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unwrap-call")
        .arg("--file")
        .arg(&el_file)
        .arg("--path")
        .arg("0")
        .arg("--function")
        .arg("with-current-buffer")
        .arg("--argument-index")
        .arg("1")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains(
            "\"replacement\": \"(message \\\"ready\\\")\"",
        ))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(el_file).expect("read unwrapped emacs lisp"),
        "(message \"ready\")\n"
    );
}

#[test]
fn cli_rejects_unwrap_call_function_mismatch() {
    let mut cmd = paredit();
    cmd.args([
        "unwrap-call",
        "--path",
        "0",
        "--function",
        "with-transaction",
    ])
    .write_stdin("(with-cache (fetch-user id))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("expected function"));
}

fn generated_unwrap_call_input(depth: usize) -> String {
    let mut expression = "seed".to_owned();
    for index in 0..depth {
        expression = format!("(step-{index} {expression})");
    }
    format!("(with-wrapper {expression} :trace)")
}

fn assert_unwrap_call_property(input: String) -> Result<(), TestCaseError> {
    let output = paredit()
        .args([
            "unwrap-call",
            "--path",
            "0",
            "--function",
            "with-wrapper",
            "--argument-index",
            "0",
            "--output",
            "json",
        ])
        .write_stdin(input)
        .output()
        .map_err(|err| TestCaseError::fail(format!("run paredit: {err}")))?;

    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .map_err(|err| TestCaseError::fail(format!("parse json: {err}")))?;
    prop_assert_eq!(report["function"].as_str(), Some("with-wrapper"));
    prop_assert_eq!(report["argumentIndex"].as_u64(), Some(0));
    prop_assert_eq!(report["callArgumentCount"].as_u64(), Some(2));
    prop_assert_eq!(report["changed"].as_bool(), Some(true));

    let rewritten = report["rewritten"].as_str().unwrap_or_default();
    let check_output = paredit()
        .arg("check")
        .write_stdin(rewritten.to_owned())
        .output()
        .map_err(|err| TestCaseError::fail(format!("run check: {err}")))?;
    prop_assert!(
        check_output.status.success(),
        "check stderr={}",
        String::from_utf8_lossy(&check_output.stderr)
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    #[test]
    fn cli_unwrap_call_preserves_parseability_for_generated_wrappers(depth in 1usize..8) {
        assert_unwrap_call_property(generated_unwrap_call_input(depth))?;
    }
}
