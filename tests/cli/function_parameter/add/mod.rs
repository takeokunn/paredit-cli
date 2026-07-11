use super::*;

use proptest::test_runner::TestCaseError;

fn run_add_function_parameter(args: &[&str], stdin: &str) -> std::process::Output {
    paredit()
        .arg("refactor")
        .args(args)
        .write_stdin(stdin)
        .output()
        .expect("run add-function-parameter")
}

fn assert_add_function_parameter_success(args: &[&str], stdin: &str, stdout_needles: &[&str]) {
    let output = run_add_function_parameter(args, stdin);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in stdout_needles {
        assert!(
            stdout.contains(needle),
            "missing stdout fragment {needle:?} in {stdout}"
        );
    }
}

fn assert_add_function_parameter_failure(args: &[&str], stdin: &str, stderr_needles: &[&str]) {
    let output = run_add_function_parameter(args, stdin);
    assert!(
        !output.status.success(),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    for needle in stderr_needles {
        assert!(
            stderr.contains(needle),
            "missing stderr fragment {needle:?} in {stderr}"
        );
    }
}

mod all_calls;
mod basic;
mod common_lisp;
mod property;
mod write;
