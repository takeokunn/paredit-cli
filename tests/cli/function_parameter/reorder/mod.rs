pub(super) use super::*;

fn reorder_command() -> Command {
    let mut cmd = paredit();
    cmd.arg("reorder-function-parameters");
    cmd
}

fn push_parameter_args(cmd: &mut Command, parameters: &[&str]) {
    for parameter in parameters {
        cmd.arg("--parameter").arg(parameter);
    }
}

fn run_reorder_with_stdin(args: &[&str], parameters: &[&str], stdin: &str) -> std::process::Output {
    let mut cmd = reorder_command();
    cmd.args(args);
    push_parameter_args(&mut cmd, parameters);
    cmd.write_stdin(stdin);
    cmd.output().expect("run reorder-function-parameters")
}

fn assert_reorder_success_output(output: std::process::Output) -> String {
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

fn assert_reorder_stdout(args: &[&str], parameters: &[&str], stdin: &str, needles: &[&str]) {
    let stdout = assert_reorder_success_output(run_reorder_with_stdin(args, parameters, stdin));
    assert_stdout_contains_all(&stdout, needles);
}

fn common_lisp_reorder_args<'a>(definition_path: &'a str, call_path: &'a str) -> Vec<&'a str> {
    vec![
        "--dialect",
        "common-lisp",
        "--definition-path",
        definition_path,
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
