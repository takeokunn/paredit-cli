use super::*;

#[test]
fn cli_rejects_introduce_let_write_without_file() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--write",
    ])
    .write_stdin("(defun render () (+ (* width height) margin))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}
