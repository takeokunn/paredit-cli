use super::*;

#[test]
fn cli_rejects_extract_function_write_without_file() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "compute-sum",
        "--write",
    ])
    .write_stdin("(defun render () (+ 1 2))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}
