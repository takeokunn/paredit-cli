use super::*;

#[test]
fn cli_plans_remove_function_parameter_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "remove-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--call-path",
        "1.3",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun area (width height margin) (* width height))\n(defun render () (area 10 20 5))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"function_name\": \"area\""))
    .stdout(predicate::str::contains("\"parameter_name\": \"margin\""))
    .stdout(predicate::str::contains("\"parameter_index\": 2"))
    .stdout(predicate::str::contains(
        "\"removed_arguments\": [\n    \"5\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "(defun area (width height) (* width height))",
    ))
    .stdout(predicate::str::contains("(defun render () (area 10 20))"));
}

#[test]
fn cli_writes_remove_function_parameter_for_scheme() {
    let dir = fresh_temp_dir("remove-function-parameter");
    let scheme_file = dir.join("render.scm");
    fs::write(
        &scheme_file,
        "(define (area scale width height) (* width height))\n(define rendered (area 2 10 20))\n",
    )
    .expect("write scheme fixture");

    let mut cmd = paredit();
    cmd.arg("remove-function-parameter")
        .arg("--file")
        .arg(&scheme_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--name")
        .arg("scale")
        .arg("--call-path")
        .arg("1.2")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"function_name\": \"area\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(scheme_file).expect("read parameter-pruned scheme"),
        "(define (area width height) (* width height))\n(define rendered (area 10 20))\n"
    );
}

#[test]
fn cli_rejects_remove_function_parameter_missing_argument() {
    let mut cmd = paredit();
    cmd.args([
        "remove-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--call-path",
        "1.3",
    ])
    .write_stdin(
        "(defun area (width height margin) (* width height))\n(defun render () (area 10 20))",
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains("does not have argument"));
}
