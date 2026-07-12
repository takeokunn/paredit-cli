use super::*;

#[test]
fn merges_nested_let_star_for_emacs_lisp() {
    let output = paredit()
        .args([
            "refactor",
            "merge-nested-let-star",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(let* ((x 1)) (let* ((y (+ x 1))) (+ x y)))")
        .output()
        .expect("run merge");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("(let* ((x 1) (y (+ x 1))) (+ x y))"));
    assert!(stdout.contains("\"inner_binding_count\": 1"));
}

#[test]
fn rejects_extra_outer_body_form() {
    paredit()
        .args([
            "refactor",
            "merge-nested-let-star",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(let* ((x 1)) (print x) (let* ((y 2)) y))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "outer body to contain only one form",
        ));
}
