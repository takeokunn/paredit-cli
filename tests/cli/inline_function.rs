use super::*;

#[test]
fn cli_plans_inline_function_with_parameters() {
    let mut cmd = paredit();
    cmd.args([
        "inline-function",
        "--definition-path",
        "0",
        "--call-path",
        "1.3",
        "--output",
        "json",
    ])
    .write_stdin("(defun area (width height) (* width height))\n(defun render () (area 10 20))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"function_name\": \"area\""))
    .stdout(predicate::str::contains("\"definition_path\": \"0\""))
    .stdout(predicate::str::contains("\"call_path\": \"1.3\""))
    .stdout(predicate::str::contains("\"replacement\": \"(* 10 20)\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains("(defun render () (* 10 20))"));
}

#[test]
fn cli_writes_inline_function_and_removes_definition() {
    let dir = fresh_temp_dir("inline-function");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("inline-function")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--call-path")
        .arg("1.3")
        .arg("--remove-definition")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_removed\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read inlined lisp"),
        "(defun render () (* 10 20))\n"
    );
}

#[test]
fn cli_plans_inline_function_with_all_calls() {
    let mut cmd = paredit();
    cmd.args([
        "inline-function",
        "--definition-path",
        "0",
        "--all-calls",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun area (width height) (* width height))\n\
         (defun render () (area 10 20))\n\
         (defun summarize () (+ (area 3 4) 1))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"all_calls\": true"))
    .stdout(predicate::str::contains("\"call_paths\": ["))
    .stdout(predicate::str::contains("\"1.3\""))
    .stdout(predicate::str::contains("\"2.3.1\""))
    .stdout(predicate::str::contains("\"replacement\": \"(* 10 20)\""))
    .stdout(predicate::str::contains("\"replacement\": \"(* 3 4)\""))
    .stdout(predicate::str::contains("(defun render () (* 10 20))"))
    .stdout(predicate::str::contains(
        "(defun summarize () (+ (* 3 4) 1))",
    ));
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

    let mut cmd = paredit();
    cmd.arg("inline-function")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--all-calls")
        .arg("--remove-definition")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_removed\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read inlined lisp"),
        "(defun render () (* 10 20))\n(defun summarize () (+ (* 3 4) 1))\n"
    );
}

#[test]
fn cli_rejects_inline_function_all_calls_with_explicit_call_path() {
    let mut cmd = paredit();
    cmd.args([
        "inline-function",
        "--definition-path",
        "0",
        "--call-path",
        "1.3",
        "--all-calls",
    ])
    .write_stdin("(defun area (width height) (* width height))\n(defun render () (area 10 20))")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "inline-function accepts either --all-calls or repeated --call-path, not both",
    ));
}

#[test]
fn cli_rejects_inline_function_duplicate_evaluation_without_flag() {
    let mut cmd = paredit();
    cmd.args([
        "inline-function",
        "--definition-path",
        "0",
        "--call-path",
        "1.3",
    ])
    .write_stdin("(defun twice (x) (+ x x))\n(defun render () (twice (expensive)))")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "inline-function would duplicate argument",
    ));
}
