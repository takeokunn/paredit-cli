use super::*;

#[test]
fn converts_independent_let_in_both_dialects() {
    for dialect in ["common-lisp", "emacs-lisp"] {
        let output = paredit()
            .args([
                "refactor",
                "convert-let-to-let-star",
                "--dialect",
                dialect,
                "--path",
                "0",
            ])
            .write_stdin("(let ((x 1) (y 2)) (+ x y))")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .contains("(let* ((x 1) (y 2)) (+ x y))"));
    }
}

#[test]
fn rejects_initializer_dependency() {
    paredit()
        .args([
            "refactor",
            "convert-let-to-let-star",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(let ((x 1) (y (+ x 2))) y)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("references earlier binding 'x'"));
}
