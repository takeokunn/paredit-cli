use super::*;

#[test]
fn plans_safe_immediate_lambda_inline() {
    let output = paredit()
        .args([
            "refactor",
            "inline-lambda",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("((lambda (x y) (+ x y)) 1 2)")
        .output()
        .expect("run inline-lambda");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("decode stdout");
    assert!(stdout.contains("\"name\": \"x\""));
    assert!(stdout.contains("(let ((x 1) (y 2)) (+ x y))"));
}

#[test]
fn rejects_return_that_depends_on_lambda_boundary() {
    paredit()
        .args([
            "refactor",
            "inline-lambda",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("((lambda () (return 1)))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("function boundary"));
}
