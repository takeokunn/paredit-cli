use super::*;

#[test]
fn cli_requires_file_for_inline_function_writes() {
    assert_inline_failure(
        &["--definition-path", "0", "--call-path", "1.3", "--write"],
        None,
        &["--write requires --file"],
    );
}

#[test]
fn cli_plans_inline_function_with_parameters() {
    assert_inline_success(
        &[
            "--definition-path",
            "0",
            "--call-path",
            "1.3",
            "--output",
            "json",
        ],
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))",
        &[
            "\"function_name\": \"area\"",
            "\"definition_path\": \"0\"",
            "\"call_path\": \"1.3\"",
            "\"replacement\": \"(* 10 20)\"",
            "\"reference_count\": 1",
            "(defun render () (* 10 20))",
        ],
        &[],
    );
}

#[test]
fn cli_rejects_inline_function_duplicate_evaluation_without_flag() {
    assert_inline_failure(
        &["--definition-path", "0", "--call-path", "1.3"],
        Some("(defun twice (x) (+ x x))\n(defun render () (twice (expensive)))"),
        &["inline-function would duplicate argument"],
    );
}

#[test]
fn cli_rejects_inline_function_all_calls_with_explicit_call_path() {
    assert_inline_failure(
        &[
            "--definition-path",
            "0",
            "--call-path",
            "1.3",
            "--all-calls",
        ],
        Some("(defun area (width height) (* width height))\n(defun render () (area 10 20))"),
        &["inline-function accepts either --all-calls or repeated --call-path, not both"],
    );
}
