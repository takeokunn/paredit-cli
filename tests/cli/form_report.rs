use super::*;

fn generated_form_report_input(depth: usize) -> String {
    let mut input = "(defun generated (x)".to_owned();
    for index in 0..depth {
        input.push_str(&format!(" (step-{index} x"));
    }
    input.push_str(" x");
    for _ in 0..depth {
        input.push(')');
    }
    input.push(')');
    input
}

fn assert_form_report_property(input: String) -> Result<(), TestCaseError> {
    let output = paredit()
        .args([
            "form-report",
            "--dialect",
            "common-lisp",
            "--path",
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
    prop_assert_eq!(report["kind"].as_str(), Some("list"));
    prop_assert_eq!(report["span"]["start"].as_u64(), Some(0));
    prop_assert_eq!(report["head"].as_str(), Some("defun"));
    prop_assert_eq!(report["definitionLike"].as_bool(), Some(true));
    prop_assert!(report["atomCount"].as_u64().unwrap_or_default() >= 4);
    prop_assert!(report["listCount"].as_u64().unwrap_or_default() >= 2);
    prop_assert!(report["maxDepth"].as_u64().unwrap_or_default() >= 2);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    #[test]
    fn cli_form_report_preserves_agent_schema_for_generated_forms(depth in 1usize..8) {
        assert_form_report_property(generated_form_report_input(depth))?;
    }

}

#[test]
fn cli_prints_agent_report_json() {
    let mut cmd = paredit();
    cmd.args([
        "agent-report",
        "--file",
        "tests/fixtures/system.asd",
        "--output",
        "json",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"label\": \"common-lisp\""))
    .stdout(predicate::str::contains("\"definitionLike\": true"));
}
