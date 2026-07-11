use super::*;
use serde_json::Value;
use std::path::Path;

mod policy;
mod single_file;
mod workspace;

struct PlanOutput {
    stdout: String,
    json: Value,
}

fn run_refactor_plan_json(path: &Path, symbol: &str, operation: &str) -> PlanOutput {
    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "plan"])
        .arg("--symbol")
        .arg(symbol)
        .arg("--operation")
        .arg(operation)
        .arg("--output")
        .arg("json")
        .arg(path)
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    let json = parse_cli_json(stdout.as_bytes());
    PlanOutput { stdout, json }
}

fn run_workspace_refactor_plan_json(
    path: &Path,
    symbol: &str,
    operation: &str,
    extra_args: &[&str],
) -> PlanOutput {
    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "workspace-plan"])
        .arg("--symbol")
        .arg(symbol)
        .arg("--operation")
        .arg(operation)
        .args(extra_args)
        .arg("--output")
        .arg("json")
        .arg(path)
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    let json = parse_cli_json(stdout.as_bytes());
    PlanOutput { stdout, json }
}

fn assert_json_string_field(json: &Value, field: &str, expected: &str) {
    assert_eq!(
        json.get(field).and_then(Value::as_str),
        Some(expected),
        "unexpected value for {field}",
    );
}

fn assert_refactor_plan_target(
    fixture_name: &str,
    source: &str,
    symbol: &str,
    operation: &str,
    target_kind: &str,
    next_action: &str,
    command_fragment: &str,
) {
    let dir = fresh_temp_dir(fixture_name);
    let file = dir.join("core.lisp");
    write_fixture(&file, source);

    let output = run_refactor_plan_json(&file, symbol, operation);

    assert_json_string_field(&output.json, "operation", operation);
    assert_json_string_field(&output.json, "symbol", symbol);
    assert_json_string_field(&output.json, "target_kind", target_kind);
    assert_eq!(
        output
            .json
            .pointer("/decision/status")
            .and_then(Value::as_str),
        Some("ready")
    );
    assert_eq!(
        output
            .json
            .pointer("/decision/next_action")
            .and_then(Value::as_str),
        Some(next_action)
    );
    assert!(
        output.stdout.contains(command_fragment),
        "stdout did not contain expected command fragment: {command_fragment}\n{}",
        output.stdout
    );
}

fn assert_workspace_plan_target(
    fixture_name: &str,
    source: &str,
    symbol: &str,
    operation: &str,
    target_kind: &str,
    next_action: &str,
    command_fragment: &str,
) {
    let dir = fresh_temp_dir(fixture_name);
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let file = src_dir.join("core.lisp");
    write_fixture(&file, source);

    let output = run_workspace_refactor_plan_json(&dir, symbol, operation, &[]);

    assert_json_string_field(&output.json, "operation", operation);
    assert_json_string_field(&output.json, "symbol", symbol);
    assert_json_string_field(&output.json, "target_kind", target_kind);
    assert!(
        output.json.get("decision").is_some(),
        "workspace plan must include decision"
    );
    assert_eq!(
        output
            .json
            .pointer("/decision/status")
            .and_then(Value::as_str),
        Some("ready")
    );
    assert_eq!(
        output
            .json
            .pointer("/decision/next_action")
            .and_then(Value::as_str),
        Some(next_action)
    );
    assert_eq!(
        output
            .json
            .pointer("/decision/safe_to_automate")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert!(
        output.stdout.contains(command_fragment),
        "stdout did not contain expected command fragment: {command_fragment}\n{}",
        output.stdout
    );
}
