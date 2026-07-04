use super::*;

#[test]
fn cli_plans_swap_function_parameters_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "swap-function-parameters",
        "--definition-path",
        "0",
        "--left-name",
        "scale",
        "--right-name",
        "height",
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
    .stdout(predicate::str::contains("\"left_name\": \"scale\""))
    .stdout(predicate::str::contains("\"right_name\": \"height\""))
    .stdout(predicate::str::contains("\"left_index\": 0"))
    .stdout(predicate::str::contains("\"right_index\": 2"))
    .stdout(predicate::str::contains("\"left\": \"2\""))
    .stdout(predicate::str::contains("\"right\": \"20\""))
    .stdout(predicate::str::contains(
        "(defun area (height width scale) (* scale width height))",
    ))
    .stdout(predicate::str::contains("(defun render () (area 20 10 2))"));
}

#[test]
fn cli_writes_swap_function_parameters_with_all_calls() {
    let dir = fresh_temp_dir("swap-function-parameters-all-calls");
    let common_lisp_file = dir.join("render.lisp");
    fs::write(
        &common_lisp_file,
        "(defun area (scale width height) (* scale width height))\n(defun a () (area 2 10 20))\n(defun b () (list (area 3 4 5)))\n",
    )
    .expect("write common lisp fixture");

    let mut cmd = paredit();
    cmd.arg("swap-function-parameters")
        .arg("--file")
        .arg(&common_lisp_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--left-name")
        .arg("scale")
        .arg("--right-name")
        .arg("height")
        .arg("--all-calls")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"all_calls\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(common_lisp_file).expect("read parameter-swapped common lisp"),
        "(defun area (height width scale) (* scale width height))\n(defun a () (area 20 10 2))\n(defun b () (list (area 5 4 3)))\n"
    );
}

#[test]
fn cli_rejects_swap_function_parameters_when_call_argument_is_missing() {
    let mut cmd = paredit();
    cmd.args([
        "swap-function-parameters",
        "--definition-path",
        "0",
        "--left-name",
        "width",
        "--right-name",
        "height",
        "--call-path",
        "1.3",
    ])
    .write_stdin("(defun area (width height) (* width height))\n(defun render () (area 10))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("has 1 arguments"));
}
