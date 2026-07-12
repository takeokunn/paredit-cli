use super::*;

#[test]
fn replaces_nested_multi_body_let_with_progn() {
    let output = paredit()
        .args([
            "refactor",
            "eliminate-empty-binding-form",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0.2",
        ])
        .write_stdin("(if ok (let () one two) nil)")
        .output()
        .expect("run elimination");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("(if ok (progn one two) nil)"));
    assert!(stdout.contains("\"body_form_count\": 2"));
    assert!(stdout.contains("\"introduced_progn\": true"));
}

#[test]
fn rejects_top_level_empty_binding_form() {
    paredit()
        .args([
            "refactor",
            "eliminate-empty-binding-form",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(let () one)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("top-level"));
}
