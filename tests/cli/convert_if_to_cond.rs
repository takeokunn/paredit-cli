use super::*;

#[test]
fn converts_common_lisp_if_with_else() {
    let output = paredit()
        .args([
            "refactor",
            "convert-if-to-cond",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(if ready (run) (wait))")
        .output()
        .expect("run conversion");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("(cond (ready (run)) ((quote t) (wait)))"));
    assert!(stdout.contains("\"has_else\": true"));
}

#[test]
fn converts_emacs_lisp_if_without_else() {
    let output = paredit()
        .args([
            "refactor",
            "convert-if-to-cond",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(if ready (message \"ok\"))")
        .output()
        .expect("run conversion");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("(cond (ready (message \\\"ok\\\")))"));
}

#[test]
fn rejects_emacs_lisp_multi_form_else() {
    paredit()
        .args([
            "refactor",
            "convert-if-to-cond",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(if ready (run) (wait) (cleanup))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires (if test then [else])"));
}
