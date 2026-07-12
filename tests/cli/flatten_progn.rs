use super::*;

#[test]
fn flattens_nested_progn_for_emacs_lisp() {
    let output = paredit()
        .args([
            "refactor",
            "flatten-progn",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0.1",
        ])
        .write_stdin("(message (progn one (progn two three) four))")
        .output()
        .expect("run flatten");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("(message (progn one two three four))"));
    assert!(stdout.contains("\"nested_count\": 1"));
}

#[test]
fn rejects_top_level_progn() {
    paredit()
        .args([
            "refactor",
            "flatten-progn",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(progn one two)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("top-level progn"));
}
