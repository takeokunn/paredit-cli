use super::*;

#[test]
fn plans_symbol_macro_inline() {
    let output = paredit()
        .args([
            "refactor",
            "inline-symbol-macro",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(symbol-macrolet ((x (+ a 1))) (list x x))")
        .output()
        .expect("run inline-symbol-macro");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("decode stdout");
    assert!(stdout.contains("\"reference_count\": 2"));
    assert!(stdout.contains("(list (+ a 1) (+ a 1))"));
}

#[test]
fn rejects_symbol_macro_place_reference() {
    paredit()
        .args([
            "refactor",
            "inline-symbol-macro",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(symbol-macrolet ((x (car cell))) (setf x 1))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("mutation places"));
}
