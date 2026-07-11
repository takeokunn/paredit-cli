use super::*;
use proptest::test_runner::TestCaseError;
use std::path::PathBuf;

fn local_function_fixture_path(case_name: &str, extension: &str) -> PathBuf {
    let dir = fresh_temp_dir(case_name);
    dir.join(format!("core.{extension}"))
}

fn write_local_function_fixture(case_name: &str, extension: &str, input: &str) -> PathBuf {
    let path = local_function_fixture_path(case_name, extension);
    fs::write(&path, input).expect("write local function fixture");
    path
}

fn assert_cli_local_function_plan(
    case_name: &str,
    extension: &str,
    input: &str,
    expected_definition_count: u64,
    expected_call_count: u64,
    expected_fragment: &str,
) {
    let fixture = write_local_function_fixture(case_name, extension, input);

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "\"definitionCount\": {expected_definition_count}"
        )))
        .stdout(predicate::str::contains(format!(
            "\"callCount\": {expected_call_count}"
        )))
        .stdout(predicate::str::contains(expected_fragment));

    assert_eq!(
        fs::read_to_string(&fixture).expect("read unchanged local function fixture"),
        input
    );
}

fn assert_cli_local_function_write(
    case_name: &str,
    extension: &str,
    input: &str,
    expected_definition_count: u64,
    expected_call_count: u64,
    expected_output: &str,
) {
    let fixture = write_local_function_fixture(case_name, extension, input);

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "\"definitionCount\": {expected_definition_count}"
        )))
        .stdout(predicate::str::contains(format!(
            "\"callCount\": {expected_call_count}"
        )))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&fixture).expect("read rewritten local function fixture"),
        expected_output
    );
}

fn assert_cli_rename_local_function_property(
    from: String,
    to: String,
) -> Result<(), TestCaseError> {
    prop_assume!(from != to);

    let fixture = local_function_fixture_path("rename-local-function-cli-pbt", "lisp");
    let input = format!("(labels (({from} (x) ({from} x))) ({from} 1) {from})\n");
    fs::write(&fixture, &input)
        .map_err(|err| TestCaseError::fail(format!("write lisp fixture: {err}")))?;

    let output = paredit()
        .args([
            "rename-local-function",
            "--from",
            &from,
            "--to",
            &to,
            "--write",
        ])
        .arg(&fixture)
        .output()
        .map_err(|err| TestCaseError::fail(format!("run paredit: {err}")))?;

    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_definition_call_report(&output.stdout)?;
    prop_assert_eq!(report.definition_count, 1);
    prop_assert_eq!(report.call_count, 2);
    prop_assert_eq!(report.files.first().map(|file| file.written), Some(true));

    let rewritten = fs::read_to_string(&fixture)
        .map_err(|err| TestCaseError::fail(format!("read rewritten fixture: {err}")))?;
    let expected = format!("(labels (({to} (x) ({to} x))) ({to} 1) {from})\n");
    prop_assert_eq!(rewritten, expected);

    assert_cli_check_succeeds(&fixture)?;

    Ok(())
}

mod accessors;
mod basic_forms;
mod errors;
mod macro_expanders;
mod property;
mod setf;
