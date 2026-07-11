use super::*;
use proptest::test_runner::TestCaseError;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
struct FixtureFile {
    path: &'static str,
    contents: &'static str,
}

struct WriteCase {
    fixture_name: &'static str,
    dialect: Option<&'static str>,
    from: &'static str,
    to: &'static str,
    input_files: &'static [FixtureFile],
    expected_files: &'static [FixtureFile],
    expected_definition_count: u64,
    expected_call_count: u64,
}

struct PlanCase {
    fixture_name: &'static str,
    from: &'static str,
    to: &'static str,
    input_files: &'static [FixtureFile],
    stdout_needles: &'static [&'static str],
    unchanged_files: &'static [FixtureFile],
}

fn fixture_path(dir: &Path, relative_path: &str) -> PathBuf {
    dir.join(relative_path)
}

fn write_fixture_files(dir: &Path, files: &[FixtureFile]) {
    for file in files {
        let path = fixture_path(dir, file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent directory");
        }
        fs::write(&path, file.contents).expect("write fixture");
    }
}

fn collect_fixture_paths(dir: &Path, files: &[FixtureFile]) -> Vec<PathBuf> {
    files
        .iter()
        .map(|file| fixture_path(dir, file.path))
        .collect()
}

fn run_rename_function(
    from: &str,
    to: &str,
    dialect: Option<&str>,
    write: bool,
    paths: &[PathBuf],
) -> std::process::Output {
    let mut cmd = paredit();
    cmd.arg("rename-function");
    if let Some(dialect) = dialect {
        cmd.arg("--dialect").arg(dialect);
    }
    cmd.arg("--from").arg(from).arg("--to").arg(to);
    if write {
        cmd.arg("--write");
    }
    for path in paths {
        cmd.arg(path);
    }
    cmd.output().expect("run rename-function")
}

fn assert_definition_call_report_matches(
    output: &std::process::Output,
    expected_definition_count: u64,
    expected_call_count: u64,
    expected_file_count: usize,
    expected_written: bool,
) {
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report =
        parse_definition_call_report(&output.stdout).expect("parse definition-call report");
    assert_eq!(report.definition_count, expected_definition_count);
    assert_eq!(report.call_count, expected_call_count);
    assert_eq!(report.files.len(), expected_file_count);
    assert!(
        report
            .files
            .iter()
            .all(|file| file.written == expected_written)
    );
}

fn assert_file_contents(dir: &Path, files: &[FixtureFile]) {
    for file in files {
        assert_eq!(
            fs::read_to_string(fixture_path(dir, file.path)).expect("read fixture"),
            file.contents
        );
    }
}

fn assert_write_case(case: WriteCase) {
    let dir = fresh_temp_dir(case.fixture_name);
    write_fixture_files(&dir, case.input_files);
    let paths = collect_fixture_paths(&dir, case.input_files);

    let output = run_rename_function(case.from, case.to, case.dialect, true, &paths);
    assert_definition_call_report_matches(
        &output,
        case.expected_definition_count,
        case.expected_call_count,
        case.input_files.len(),
        true,
    );
    assert_file_contents(&dir, case.expected_files);
}

fn assert_plan_case(case: PlanCase) {
    let dir = fresh_temp_dir(case.fixture_name);
    write_fixture_files(&dir, case.input_files);
    let paths = collect_fixture_paths(&dir, case.input_files);

    let output = run_rename_function(case.from, case.to, None, false, &paths);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("decode stdout");
    for needle in case.stdout_needles {
        assert!(stdout.contains(needle), "missing stdout fragment: {needle}");
    }
    assert_file_contents(&dir, case.unchanged_files);
}

fn assert_cli_rename_function_property(from: String, to: String) -> Result<(), TestCaseError> {
    prop_assume!(from != to);

    let dir = fresh_temp_dir("rename-function-cli-pbt");
    let lisp_file = dir.join("core.lisp");
    let input = format!("(defun {from} (x) (list {from} x))\n(defun caller () ({from} 1))\n");
    fs::write(&lisp_file, &input)
        .map_err(|err| TestCaseError::fail(format!("write lisp fixture: {err}")))?;

    let output = run_rename_function(&from, &to, None, true, std::slice::from_ref(&lisp_file));
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
    let expected = format!("(defun {to} (x) (list {from} x))\n(defun caller () ({to} 1))\n");
    prop_assert_eq!(rewritten, expected);

    assert_cli_check_succeeds(&lisp_file)?;

    Ok(())
}

proptest! {
    #![proptest_config(cli_proptest_config(24))]

    #[test]
    fn pbt_cli_rename_function_output_remains_parseable_and_preserves_value_refs(
        from in "[a-z][a-z0-9-]{0,8}",
        to in "[a-z][a-z0-9-]{0,8}",
    ) {
        assert_cli_rename_function_property(from, to)?;
    }
}

#[cfg(unix)]
fn assert_rollback_on_write_failure() {
    let dir = fresh_temp_dir("rename-function-rollback");
    let definition_file = dir.join("core.lisp");
    let nested_dir = dir.join("nested");
    fs::create_dir(&nested_dir).expect("create nested fixture directory");
    let caller_file = nested_dir.join("caller.lisp");
    fs::write(&definition_file, "(defun old-name (x) x)\n(old-name 1)\n")
        .expect("write definition fixture");
    fs::write(&caller_file, "(defun caller () (old-name 2))\n").expect("write caller fixture");

    let mut permissions = fs::metadata(&nested_dir)
        .expect("read nested dir metadata")
        .permissions();
    permissions.set_mode(0o555);
    fs::set_permissions(&nested_dir, permissions.clone()).expect("lock nested dir");

    let output = run_rename_function(
        "old-name",
        "new-name",
        None,
        true,
        &[definition_file.clone(), caller_file.clone()],
    );

    let mut restore_permissions = permissions;
    restore_permissions.set_mode(0o755);
    fs::set_permissions(&nested_dir, restore_permissions).expect("unlock nested dir");

    assert!(
        !output.status.success(),
        "rename-function unexpectedly succeeded"
    );
    assert_eq!(
        fs::read_to_string(&definition_file).expect("read definition after rollback"),
        "(defun old-name (x) x)\n(old-name 1)\n"
    );
    assert_eq!(
        fs::read_to_string(&caller_file).expect("read caller after rollback"),
        "(defun caller () (old-name 2))\n"
    );
}

mod failure;
mod macro_like;
mod macrolet_scope;
mod plan;
mod plan_macrolet_scope;
mod plan_symbol_macro_scope;
mod setf_and_generic;
mod setf_scope;
mod symbol_macro_scope;
mod write_basic;
