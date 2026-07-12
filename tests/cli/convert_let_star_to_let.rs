use super::*;

#[test]
fn converts_independent_let_star() {
    let output = paredit()
        .args([
            "refactor",
            "convert-let-star-to-let",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(let* ((x (first)) (y (second))) (+ x y))")
        .output()
        .expect("run conversion");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("(let ((x (first)) (y (second))) (+ x y))"));
    assert!(stdout.contains("\"binding_names\": ["));
}

#[test]
fn rejects_initializer_dependency() {
    paredit()
        .args([
            "refactor",
            "convert-let-star-to-let",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(let* ((x 1) (y (+ x 2))) y)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("references earlier binding 'x'"));
}
