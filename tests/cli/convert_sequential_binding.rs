use super::*;

#[test]
fn converts_independent_do_star() {
    let output = paredit()
        .args([
            "refactor",
            "convert-do-star-to-do",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(do* ((x 1 (1+ x)) (y 2 (1+ y))) ((> x 4) y))")
        .output()
        .expect("run conversion");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("(do ((x 1 (1+ x)) (y 2 (1+ y)))"));
}

#[test]
fn rejects_do_star_step_dependency() {
    paredit()
        .args([
            "refactor",
            "convert-do-star-to-do",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(do* ((x 1 (1+ x)) (y 2 (+ x y))) ((> y 4) y))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("step expression for 'y'"));
}

#[test]
fn converts_independent_prog_star() {
    let output = paredit()
        .args([
            "refactor",
            "convert-prog-star-to-prog",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(prog* ((x 1) (y 2)) (return (+ x y)))")
        .output()
        .expect("run conversion");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("(prog ((x 1) (y 2))"));
}

#[test]
fn rejects_prog_star_initializer_dependency() {
    paredit()
        .args([
            "refactor",
            "convert-prog-star-to-prog",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(prog* ((x 1) (y (+ x 1))) (return y))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("initializer for 'y'"));
}
