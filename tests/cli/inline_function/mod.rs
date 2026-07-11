pub(super) use super::*;

const INLINE_FUNCTION_COMMON_LISP_ARGS: &[&str] = &[
    "--dialect",
    "common-lisp",
    "--definition-path",
    "0",
    "--call-path",
    "1.1",
    "--output",
    "json",
];

fn run_inline_function(args: &[&str], stdin: Option<&str>) -> std::process::Output {
    let mut cmd = paredit();
    cmd.arg("inline-function");
    cmd.args(args);
    if let Some(stdin) = stdin {
        cmd.write_stdin(stdin);
    }
    cmd.output().expect("run inline-function")
}

fn assert_inline_success(
    args: &[&str],
    stdin: &str,
    stdout_needles: &[&str],
    stdout_absent: &[&str],
) {
    let output = run_inline_function(args, Some(stdin));
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("decode stdout");
    for needle in stdout_needles {
        assert!(stdout.contains(needle), "missing stdout fragment: {needle}");
    }
    for needle in stdout_absent {
        assert!(
            !stdout.contains(needle),
            "unexpected stdout fragment present: {needle}"
        );
    }
}

fn assert_inline_failure(args: &[&str], stdin: Option<&str>, stderr_needles: &[&str]) {
    let output = run_inline_function(args, stdin);
    assert!(
        !output.status.success(),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8(output.stderr).expect("decode stderr");
    for needle in stderr_needles {
        assert!(stderr.contains(needle), "missing stderr fragment: {needle}");
    }
}

fn inline_function_common_lisp_args(extra_args: &[&'static str]) -> Vec<&'static str> {
    let mut args = INLINE_FUNCTION_COMMON_LISP_ARGS.to_vec();
    let output_index = args
        .iter()
        .position(|arg| *arg == "--output")
        .expect("common lisp args include --output");
    args.splice(output_index..output_index, extra_args.iter().copied());
    args
}

fn assert_common_lisp_inline_success(stdin: &str, stdout_needles: &[&str], stdout_absent: &[&str]) {
    assert_inline_success(
        inline_function_common_lisp_args(&[]).as_slice(),
        stdin,
        stdout_needles,
        stdout_absent,
    );
}

fn assert_common_lisp_inline_success_with_args(
    extra_args: &[&'static str],
    stdin: &str,
    stdout_needles: &[&str],
    stdout_absent: &[&str],
) {
    assert_inline_success(
        inline_function_common_lisp_args(extra_args).as_slice(),
        stdin,
        stdout_needles,
        stdout_absent,
    );
}

fn assert_common_lisp_inline_failure(stdin: &str, stderr_needles: &[&str]) {
    assert_inline_failure(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.1",
        ],
        Some(stdin),
        stderr_needles,
    );
}

mod all_calls;
mod basic;
mod common_lisp;
mod write;
