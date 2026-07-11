pub(super) use super::*;

fn swap_command() -> Command {
    let mut cmd = paredit();
    cmd.arg("swap-function-parameters");
    cmd
}

fn run_swap_with_stdin(args: &[&str], stdin: &str) -> std::process::Output {
    let mut cmd = swap_command();
    cmd.args(args);
    cmd.write_stdin(stdin);
    cmd.output().expect("run swap-function-parameters")
}

fn assert_swap_success_output(output: std::process::Output) -> String {
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

fn assert_swap_stdout(args: &[&str], stdin: &str, needles: &[&str]) {
    let stdout = assert_swap_success_output(run_swap_with_stdin(args, stdin));
    assert_stdout_contains_all(&stdout, needles);
}

fn common_lisp_swap_args<'a>(
    definition_path: &'a str,
    left_name: &'a str,
    right_name: &'a str,
    call_path: &'a str,
) -> Vec<&'a str> {
    vec![
        "--dialect",
        "common-lisp",
        "--definition-path",
        definition_path,
        "--left-name",
        left_name,
        "--right-name",
        right_name,
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
