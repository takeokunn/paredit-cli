use super::*;

mod error;
mod plan;
mod write;

fn assert_plan_output(args: &[&str], input: &str, checks: &[&str]) {
    let mut cmd = paredit();
    cmd.arg("refactor");
    let mut assert = cmd.args(args).write_stdin(input).assert().success();
    for check in checks {
        assert = assert.stdout(predicate::str::contains(*check));
    }
}

fn assert_written_file(
    fixture_name: &str,
    file_name: &str,
    initial_source: &str,
    args: &[&str],
    checks: &[&str],
    expected_source: &str,
) {
    let dir = fresh_temp_dir(fixture_name);
    let file_path = dir.join(file_name);
    fs::write(&file_path, initial_source).expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("refactor").arg("introduce-let");
    for arg in args {
        cmd.arg(arg);
    }
    let mut assert = cmd.arg("--file").arg(&file_path).assert().success();
    for check in checks {
        assert = assert.stdout(predicate::str::contains(*check));
    }

    assert_eq!(
        fs::read_to_string(&file_path).expect("read rewritten fixture"),
        expected_source
    );
}
