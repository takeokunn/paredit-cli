use super::*;
use proptest::test_runner::TestCaseError;
use std::path::Path;

fn run_rename_macrolet(path: &Path, from: &str, to: &str, write: bool) -> std::process::Output {
    let mut cmd = paredit();
    cmd.arg("rename-macrolet")
        .arg("--from")
        .arg(from)
        .arg("--to")
        .arg(to);
    if write {
        cmd.arg("--write");
    }
    cmd.arg(path).output().expect("run rename-macrolet")
}

fn write_fixture(path: &Path, input: &str, description: &str) {
    fs::write(path, input).unwrap_or_else(|err| panic!("write {description}: {err}"));
}

fn read_fixture(path: &Path, description: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {description}: {err}"))
}

fn assert_plan_case(fixture_name: &str, file_name: &str, input: &str, expected: &str) {
    let dir = fresh_temp_dir(fixture_name);
    let file = dir.join(file_name);
    write_fixture(&file, input, "plan fixture");

    let output = run_rename_macrolet(&file, "old-name", "new-name", false);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = parse_definition_call_report(&output.stdout).expect("parse plan report");
    assert_eq!(report.definition_count, 1);
    assert_eq!(report.call_count, 1);
    assert_eq!(report.files.first().map(|entry| entry.written), Some(false));
    assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    assert_eq!(read_fixture(&file, "unchanged plan fixture"), input);
}

fn assert_write_case(
    fixture_name: &str,
    file_name: &str,
    input: &str,
    expected: &str,
    expected_call_count: u64,
) {
    let dir = fresh_temp_dir(fixture_name);
    let file = dir.join(file_name);
    write_fixture(&file, input, "write fixture");

    let output = run_rename_macrolet(&file, "old-name", "new-name", true);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = parse_definition_call_report(&output.stdout).expect("parse write report");
    assert_eq!(report.definition_count, 1);
    assert_eq!(report.call_count, expected_call_count);
    assert_eq!(report.files.first().map(|entry| entry.written), Some(true));
    assert_eq!(read_fixture(&file, "rewritten write fixture"), expected);
}

fn assert_cli_rename_macrolet_property(from: String, to: String) -> Result<(), TestCaseError> {
    prop_assume!(from != to);

    let dir = fresh_temp_dir("rename-macrolet-cli-pbt");
    let lisp_file = dir.join("core.lisp");
    let input = format!("(macrolet (({from} (x) (list {from} x))) ({from} 1) {from})\n");
    fs::write(&lisp_file, &input)
        .map_err(|err| TestCaseError::fail(format!("write lisp fixture: {err}")))?;

    let output = paredit()
        .args(["rename-macrolet", "--from", &from, "--to", &to, "--write"])
        .arg(&lisp_file)
        .output()
        .map_err(|err| TestCaseError::fail(format!("run paredit: {err}")))?;

    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_definition_call_report(&output.stdout)?;
    prop_assert_eq!(report.definition_count, 1);
    prop_assert_eq!(report.call_count, 1);
    prop_assert_eq!(report.files.first().map(|file| file.written), Some(true));

    let rewritten = fs::read_to_string(&lisp_file)
        .map_err(|err| TestCaseError::fail(format!("read rewritten fixture: {err}")))?;
    let expected = format!("(macrolet (({to} (x) (list {from} x))) ({to} 1) {from})\n");
    prop_assert_eq!(rewritten, expected);

    assert_cli_check_succeeds(&lisp_file)?;

    Ok(())
}

macro_rules! macrolet_plan_case {
    ($name:ident, $fixture:expr, $file_name:expr, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_plan_case($fixture, $file_name, $input, $expected);
        }
    };
}

macro_rules! macrolet_write_case {
    ($name:ident, $fixture:expr, $file_name:expr, $input:expr, $expected:expr, $call_count:expr) => {
        #[test]
        fn $name() {
            assert_write_case($fixture, $file_name, $input, $expected, $call_count);
        }
    };
}

mod failure;
mod plan;
mod property;
mod write;
