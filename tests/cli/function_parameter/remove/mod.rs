use super::*;

use assert_cmd::Command;
use assert_cmd::assert::Assert;

fn remove_command() -> Command {
    let mut command = paredit();
    command.arg("refactor");
    command
}

fn run_remove_with_stdin(args: &[&str], input: &str) -> std::process::Output {
    remove_command()
        .args(args)
        .write_stdin(input)
        .output()
        .expect("run remove-function-parameter")
}

fn assert_remove_success_output(args: &[&str], input: &str) -> CliRemoveFunctionParameterReport {
    let output = run_remove_with_stdin(args, input);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    parse_remove_function_parameter_report(&output.stdout).expect("parse remove report")
}

fn assert_stdout_contains_all(assert: Assert, fragments: &[&str]) {
    let assert = fragments.iter().fold(assert, |assert, fragment| {
        assert.stdout(predicate::str::contains(*fragment))
    });
    drop(assert);
}

fn assert_remove_stdout(args: &[&str], input: &str, fragments: &[&str]) {
    let mut cmd = remove_command();
    cmd.args(args).write_stdin(input);
    assert_stdout_contains_all(cmd.assert().success(), fragments);
}

fn common_lisp_remove_args<'a>(
    definition_path: &'a str,
    name: &'a str,
    call_path: &'a str,
) -> Vec<&'a str> {
    vec![
        "refactor",
        "remove-function-parameter",
        "--dialect",
        "common-lisp",
        "--definition-path",
        definition_path,
        "--name",
        name,
        "--call-path",
        call_path,
    ]
}

mod basic;
mod common_lisp;
mod failure;
mod property;
mod write;
