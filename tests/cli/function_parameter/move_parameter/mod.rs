pub(super) use super::*;

fn move_command() -> Command {
    let mut cmd = paredit();
    cmd.arg("move-function-parameter");
    cmd
}

fn run_move_with_stdin(args: &[&str], stdin: &str) -> std::process::Output {
    let mut cmd = move_command();
    cmd.args(args);
    cmd.write_stdin(stdin);
    cmd.output().expect("run move-function-parameter")
}

fn assert_move_success_output(output: std::process::Output) -> String {
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("decode stdout")
}

fn assert_stdout_contains_all(stdout: &str, needles: &[&str]) {
    for needle in needles {
        assert!(stdout.contains(needle), "missing stdout fragment: {needle}");
    }
}

fn assert_move_stdout(args: &[&str], stdin: &str, needles: &[&str]) {
    let stdout = assert_move_success_output(run_move_with_stdin(args, stdin));
    assert_stdout_contains_all(&stdout, needles);
}

fn common_lisp_move_args<'a>(
    definition_path: &'a str,
    name: &'a str,
    to_index: &'a str,
    call_path: &'a str,
) -> Vec<&'a str> {
    vec![
        "--dialect",
        "common-lisp",
        "--definition-path",
        definition_path,
        "--name",
        name,
        "--to-index",
        to_index,
        "--call-path",
        call_path,
        "--output",
        "json",
    ]
}

mod common_lisp;
mod failure;
mod property;
mod write;
