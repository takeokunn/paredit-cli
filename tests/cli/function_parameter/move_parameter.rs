use super::*;

#[test]
fn cli_plans_move_function_parameter_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "move-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "scale",
        "--to-index",
        "2",
        "--call-path",
        "1.3",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun area (scale width height) (* scale width height))\n(defun render () (area 2 10 20))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"function_name\": \"area\""))
    .stdout(predicate::str::contains("\"parameter_name\": \"scale\""))
    .stdout(predicate::str::contains("\"from_index\": 0"))
    .stdout(predicate::str::contains("\"to_index\": 2"))
    .stdout(predicate::str::contains(
        "\"moved_arguments\": [\n    \"2\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "(defun area (width height scale) (* scale width height))",
    ))
    .stdout(predicate::str::contains("(defun render () (area 10 20 2))"));
}

#[test]
fn cli_writes_move_function_parameter_for_scheme() {
    let dir = fresh_temp_dir("move-function-parameter");
    let scheme_file = dir.join("render.scm");
    fs::write(
        &scheme_file,
        "(define (area width height scale) (* scale width height))\n(define rendered (area 10 20 2))\n",
    )
    .expect("write scheme fixture");

    let mut cmd = paredit();
    cmd.arg("move-function-parameter")
        .arg("--file")
        .arg(&scheme_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--name")
        .arg("scale")
        .arg("--to-index")
        .arg("0")
        .arg("--call-path")
        .arg("1.2")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"function_name\": \"area\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(scheme_file).expect("read parameter-reordered scheme"),
        "(define (area scale width height) (* scale width height))\n(define rendered (area 2 10 20))\n"
    );
}

#[test]
fn cli_writes_move_function_parameter_with_all_calls() {
    let dir = fresh_temp_dir("move-function-parameter-all-calls");
    let common_lisp_file = dir.join("render.lisp");
    fs::write(
        &common_lisp_file,
        "(defun area (scale width height) (* scale width height))\n(defun a () (area 2 10 20))\n(defun b () (list (area 3 4 5)))\n",
    )
    .expect("write common lisp fixture");

    let mut cmd = paredit();
    cmd.arg("move-function-parameter")
        .arg("--file")
        .arg(&common_lisp_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--name")
        .arg("scale")
        .arg("--to-index")
        .arg("2")
        .arg("--all-calls")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"all_calls\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(common_lisp_file).expect("read parameter-reordered common lisp"),
        "(defun area (width height scale) (* scale width height))\n(defun a () (area 10 20 2))\n(defun b () (list (area 4 5 3)))\n"
    );
}

#[test]
fn cli_rejects_move_function_parameter_target_out_of_bounds() {
    let mut cmd = paredit();
    cmd.args([
        "move-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "width",
        "--to-index",
        "2",
        "--call-path",
        "1.3",
    ])
    .write_stdin("(defun area (width height) (* width height))\n(defun render () (area 10 20))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("target index"));
}
