use super::*;

#[test]
fn cli_writes_inline_function_and_removes_definition() {
    let dir = fresh_temp_dir("inline-function");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))\n",
    )
    .expect("write lisp fixture");

    let output = run_inline_function(
        &[
            "--file",
            lisp_file.to_str().expect("fixture path utf-8"),
            "--definition-path",
            "0",
            "--call-path",
            "1.3",
            "--remove-definition",
            "--write",
        ],
        None,
    );
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("decode stdout");
    assert!(stdout.contains("\"definition_removed\": true"));
    assert!(stdout.contains("\"written\": true"));
    assert_eq!(
        fs::read_to_string(lisp_file).expect("read inlined lisp"),
        "(defun render () (* 10 20))\n"
    );
}

#[test]
fn cli_writes_inline_function_all_calls_and_removes_definition() {
    let dir = fresh_temp_dir("inline-function-all-calls");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun area (width height) (* width height))\n\
         (defun render () (area 10 20))\n\
         (defun summarize () (+ (area 3 4) 1))\n",
    )
    .expect("write lisp fixture");

    let output = run_inline_function(
        &[
            "--file",
            lisp_file.to_str().expect("fixture path utf-8"),
            "--definition-path",
            "0",
            "--all-calls",
            "--remove-definition",
            "--write",
        ],
        None,
    );
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("decode stdout");
    assert!(stdout.contains("\"definition_removed\": true"));
    assert!(stdout.contains("\"written\": true"));
    assert_eq!(
        fs::read_to_string(lisp_file).expect("read inlined lisp"),
        "(defun render () (* 10 20))\n(defun summarize () (+ (* 3 4) 1))\n"
    );
}
