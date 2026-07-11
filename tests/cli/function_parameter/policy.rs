use super::*;

#[test]
fn cli_requires_file_for_add_function_parameter_writes() {
    let mut cmd = paredit();
    cmd.args([
        "add-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--argument",
        "5",
        "--call-path",
        "1.3",
        "--write",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_requires_file_for_move_function_parameter_writes() {
    let mut cmd = paredit();
    cmd.args([
        "move-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--to-index",
        "0",
        "--call-path",
        "1.3",
        "--write",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_requires_file_for_swap_function_parameters_writes() {
    let mut cmd = paredit();
    cmd.args([
        "swap-function-parameters",
        "--definition-path",
        "0",
        "--left-name",
        "margin",
        "--right-name",
        "width",
        "--call-path",
        "1.3",
        "--write",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_requires_file_for_reorder_function_parameters_writes() {
    let mut cmd = paredit();
    cmd.args([
        "reorder-function-parameters",
        "--definition-path",
        "0",
        "--parameter",
        "margin",
        "--parameter",
        "width",
        "--parameter",
        "height",
        "--call-path",
        "1.3",
        "--write",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_requires_file_for_remove_function_parameter_writes() {
    let mut cmd = paredit();
    cmd.args([
        "remove-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--call-path",
        "1.3",
        "--write",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_rejects_function_parameter_all_calls_with_explicit_call_path() {
    let mut cmd = paredit();
    cmd.args([
        "remove-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--all-calls",
        "--call-path",
        "1.3",
    ])
    .write_stdin(
        "(defun area (width height margin) (* width height))\n(defun render () (area 10 20 5))",
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "either --all-calls or repeated --call-path",
    ));
}
