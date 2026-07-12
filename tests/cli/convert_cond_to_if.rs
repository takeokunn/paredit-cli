use super::*;

#[test]
fn converts_common_lisp_cond_to_nested_if() {
    let output = paredit()
        .args([
            "refactor",
            "convert-cond-to-if",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(cond (a b) (c d))")
        .output()
        .expect("run conversion");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("(if a b (if c d))"));
    assert!(stdout.contains("\"clause_count\": 2"));
}

#[test]
fn converts_single_emacs_lisp_clause() {
    let output = paredit()
        .args([
            "refactor",
            "convert-cond-to-if",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(cond (ready yes))")
        .output()
        .expect("run conversion");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("(if ready yes)"));
}

#[test]
fn rejects_clause_with_multiple_consequents() {
    paredit()
        .args([
            "refactor",
            "convert-cond-to-if",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(cond (ready one two))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "each clause to contain exactly test and consequent",
        ));
}
