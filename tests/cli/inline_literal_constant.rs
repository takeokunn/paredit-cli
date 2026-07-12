use super::*;

#[test]
fn plans_literal_constant_inline() {
    let output = paredit()
        .args([
            "refactor",
            "inline-literal-constant",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(defconstant +answer+ 42)\n(defun answer () +answer+)")
        .output()
        .expect("run inline-literal-constant");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("decode stdout");
    assert!(stdout.contains("\"constant_name\": \"+answer+\""));
    assert!(stdout.contains("\"reference_count\": 1"));
    assert!(stdout.contains("(defun answer () 42)"));
}

#[test]
fn rejects_mutable_literal_constant() {
    paredit()
        .args([
            "refactor",
            "inline-literal-constant",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(defconstant +items+ '(1 2))\n(print +items+)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("self-evaluating literals"));
}
