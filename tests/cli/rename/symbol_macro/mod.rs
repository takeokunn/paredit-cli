use super::*;
use proptest::test_runner::TestCaseError;
use std::path::Path;

fn run_rename_symbol_macro(
    paths: &[&Path],
    from: &str,
    to: &str,
    write: bool,
) -> std::process::Output {
    let mut cmd = paredit();
    cmd.arg("rename-symbol-macro")
        .arg("--from")
        .arg(from)
        .arg("--to")
        .arg(to);
    if write {
        cmd.arg("--write");
    }
    for path in paths {
        cmd.arg(path);
    }
    cmd.output().expect("run rename-symbol-macro")
}

fn write_fixture(path: &Path, input: &str, description: &str) {
    fs::write(path, input).unwrap_or_else(|err| panic!("write {description}: {err}"));
}

fn read_fixture(path: &Path, description: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {description}: {err}"))
}

fn assert_symbol_macro_report(
    output: &std::process::Output,
    expected_definition_count: u64,
    expected_reference_count: u64,
    expected_written: bool,
) {
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = parse_definition_reference_report(&output.stdout)
        .expect("parse definition-reference report");
    assert_eq!(report.definition_count, expected_definition_count);
    assert_eq!(report.reference_count, expected_reference_count);
    assert!(
        report
            .files
            .iter()
            .all(|file| file.written == expected_written)
    );
}

fn assert_plan_case(
    fixture_name: &str,
    input: &str,
    expected_output_fragment: &str,
    expected_definition_count: u64,
    expected_reference_count: u64,
) {
    let dir = fresh_temp_dir(fixture_name);
    let lisp_file = dir.join("core.lisp");
    write_fixture(&lisp_file, input, "plan fixture");

    let output = run_rename_symbol_macro(&[lisp_file.as_path()], "old-name", "new-name", false);
    assert_symbol_macro_report(
        &output,
        expected_definition_count,
        expected_reference_count,
        false,
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(expected_output_fragment));
    assert_eq!(read_fixture(&lisp_file, "unchanged plan fixture"), input);
}

fn assert_write_case(
    fixture_name: &str,
    input: &str,
    expected: &str,
    expected_definition_count: u64,
    expected_reference_count: u64,
) {
    let dir = fresh_temp_dir(fixture_name);
    let lisp_file = dir.join("core.lisp");
    write_fixture(&lisp_file, input, "write fixture");

    let output = run_rename_symbol_macro(&[lisp_file.as_path()], "old-name", "new-name", true);
    assert_symbol_macro_report(
        &output,
        expected_definition_count,
        expected_reference_count,
        true,
    );
    assert_eq!(
        read_fixture(&lisp_file, "rewritten write fixture"),
        expected
    );
}

fn assert_cli_rename_symbol_macro_property(from: String, to: String) -> Result<(), TestCaseError> {
    prop_assume!(from != to);

    let dir = fresh_temp_dir("rename-symbol-macro-cli-pbt");
    let lisp_file = dir.join("core.lisp");
    let input = format!(
        "(define-symbol-macro {from} current-user) (list {from} ({from} 1) (setf {from} 2))\n"
    );
    fs::write(&lisp_file, &input)
        .map_err(|err| TestCaseError::fail(format!("write lisp fixture: {err}")))?;

    let output = run_rename_symbol_macro(&[lisp_file.as_path()], &from, &to, true);
    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_definition_reference_report(&output.stdout)?;
    prop_assert_eq!(report.definition_count, 1);
    prop_assert_eq!(report.reference_count, 2);
    prop_assert_eq!(report.files.first().map(|file| file.written), Some(true));

    let rewritten = fs::read_to_string(&lisp_file)
        .map_err(|err| TestCaseError::fail(format!("read rewritten fixture: {err}")))?;
    let expected =
        format!("(define-symbol-macro {to} current-user) (list {to} ({from} 1) (setf {to} 2))\n");
    prop_assert_eq!(rewritten, expected);

    assert_cli_check_succeeds(&lisp_file)?;

    Ok(())
}

mod failure;
mod plan;
mod property;
mod shadowing;
mod write;
