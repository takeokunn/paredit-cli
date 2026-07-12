use super::*;

#[test]
fn plans_safe_single_flet_inline() {
    let output = paredit()
        .args([
            "refactor",
            "inline-local-function",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(flet ((sum (x y) (+ x y))) (sum 1 2))")
        .output()
        .expect("run inline-local-function");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("decode stdout");
    assert!(stdout.contains("\"function_name\": \"sum\""));
    assert!(stdout.contains("(let ((x 1) (y 2)) (+ x y))"));
}

#[test]
fn rejects_unsafe_duplicate_parameter_reference() {
    paredit()
        .args([
            "refactor",
            "inline-local-function",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(flet ((f (x) (+ x x))) (f (effect)))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("referenced exactly once"));
}
