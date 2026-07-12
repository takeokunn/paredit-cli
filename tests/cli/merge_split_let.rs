use super::*;

#[test]
fn merges_safe_parallel_lets() {
    paredit()
        .args([
            "refactor",
            "merge-nested-let",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(let ((x 1)) (let ((y 2)) (+ x y)))")
        .assert()
        .success()
        .stdout(predicate::str::contains("(let ((x 1) (y 2)) (+ x y))"));
}

#[test]
fn merge_rejects_initializer_dependency() {
    paredit()
        .args([
            "refactor",
            "merge-nested-let",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(let ((x 1)) (let ((y (+ x 1))) y))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("references outer binding"));
}

#[test]
fn splits_safe_parallel_let() {
    paredit()
        .args([
            "refactor",
            "split-let",
            "--dialect",
            "emacs-lisp",
            "--path",
            "0",
            "--binding-index",
            "1",
        ])
        .write_stdin("(let ((x 1) (y 2)) (+ x y))")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(let ((x 1)) (let ((y 2)) (+ x y)))",
        ));
}

#[test]
fn split_rejects_free_reference_capture() {
    paredit()
        .args([
            "refactor",
            "split-let",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--binding-index",
            "1",
        ])
        .write_stdin("(let ((x 1) (y (+ x 1))) y)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("would capture reference"));
}
