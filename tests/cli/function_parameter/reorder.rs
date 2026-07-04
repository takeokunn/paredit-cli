use super::*;

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "reorder-function-parameters",
        "--definition-path",
        "0",
        "--parameter",
        "height",
        "--parameter",
        "scale",
        "--parameter",
        "width",
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
    .stdout(predicate::str::contains(
        "\"old_parameter_order\": [\n    \"scale\",\n    \"width\",\n    \"height\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"new_parameter_order\": [\n    \"height\",\n    \"scale\",\n    \"width\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"reordered_arguments\": [\n    [\n      \"20\",\n      \"2\",\n      \"10\"\n    ]\n  ]",
    ))
    .stdout(predicate::str::contains(
        "(defun area (height scale width) (* scale width height))",
    ))
    .stdout(predicate::str::contains("(defun render () (area 20 2 10))"));
}

#[test]
fn cli_plans_reorder_function_parameters_for_common_lisp_defmethod() {
    let mut cmd = paredit();
    cmd.args([
        "reorder-function-parameters",
        "--dialect",
        "common-lisp",
        "--definition-path",
        "0",
        "--parameter",
        "style",
        "--parameter",
        "node",
        "--parameter",
        "stream",
        "--call-path",
        "1",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defmethod render :around ((node widget) stream style) (draw node stream style))\n(render thing out :fancy)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"function_name\": \"render\""))
    .stdout(predicate::str::contains(
        "\"old_parameter_order\": [\n    \"node\",\n    \"stream\",\n    \"style\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "(defmethod render :around (style (node widget) stream)",
    ))
    .stdout(predicate::str::contains("(render :fancy thing out)"));
}

#[test]
fn cli_writes_reorder_function_parameters_with_all_calls() {
    let dir = fresh_temp_dir("reorder-function-parameters-all-calls");
    let common_lisp_file = dir.join("render.lisp");
    fs::write(
        &common_lisp_file,
        "(defun area (scale width height) (* scale width height))\n(defun a () (area 2 10 20))\n(defun b () (list (area 3 4 5)))\n",
    )
    .expect("write common lisp fixture");

    let mut cmd = paredit();
    cmd.arg("reorder-function-parameters")
        .arg("--file")
        .arg(&common_lisp_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--parameter")
        .arg("height")
        .arg("--parameter")
        .arg("scale")
        .arg("--parameter")
        .arg("width")
        .arg("--all-calls")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"all_calls\": true"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(common_lisp_file).expect("read parameter-reordered common lisp"),
        "(defun area (height scale width) (* scale width height))\n(defun a () (area 20 2 10))\n(defun b () (list (area 5 3 4)))\n"
    );
}

#[test]
fn cli_rejects_reorder_function_parameters_when_call_argument_is_missing() {
    let mut cmd = paredit();
    cmd.args([
        "reorder-function-parameters",
        "--definition-path",
        "0",
        "--parameter",
        "height",
        "--parameter",
        "width",
        "--call-path",
        "1.3",
    ])
    .write_stdin("(defun area (width height) (* width height))\n(defun render () (area 10))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("has 1 arguments"));
}
