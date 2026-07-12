use super::*;

#[test]
fn splits_let_star_at_requested_binding_for_common_lisp() {
    let output = paredit()
        .args([
            "refactor",
            "split-let-star",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--binding-index",
            "1",
        ])
        .write_stdin("(let* ((x 1) (y (+ x 1))) (+ x y))")
        .output()
        .expect("run split");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("(let* ((x 1)) (let* ((y (+ x 1))) (+ x y)))"));
    assert!(stdout.contains("\"outer_binding_count\": 1"));
    assert!(stdout.contains("\"inner_binding_count\": 1"));
}

#[test]
fn rejects_binding_index_at_end() {
    paredit()
        .args([
            "refactor",
            "split-let-star",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0",
            "--binding-index",
            "2",
        ])
        .write_stdin("(let* ((x 1) (y 2)) (+ x y))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("between 1 and 1"));
}
