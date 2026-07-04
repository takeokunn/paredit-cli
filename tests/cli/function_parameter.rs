use super::*;

#[test]
fn cli_plans_add_function_parameter_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "add-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--argument",
        "5",
        "--call-path",
        "1.3",
        "--output",
        "json",
    ])
    .write_stdin("(defun area (width height) (* width height))\n(defun render () (area 10 20))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"function_name\": \"area\""))
    .stdout(predicate::str::contains("\"parameter_name\": \"margin\""))
    .stdout(predicate::str::contains("\"argument\": \"5\""))
    .stdout(predicate::str::contains("\"insert\": \"end\""))
    .stdout(predicate::str::contains(
        "(defun area (width height margin) (* width height))",
    ))
    .stdout(predicate::str::contains("(defun render () (area 10 20 5))"));
}

#[test]
fn cli_plans_add_function_parameter_with_all_calls() {
    let mut cmd = paredit();
    cmd.args([
        "add-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--argument",
        "5",
        "--all-calls",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))\n(defun total () (+ (area 3 4) 1))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"all_calls\": true"))
    .stdout(predicate::str::contains(
        "\"call_paths\": [\n    \"1.3\",\n    \"2.3.1\"\n  ]",
    ))
    .stdout(predicate::str::contains("(defun render () (area 10 20 5))"))
    .stdout(predicate::str::contains(
        "(defun total () (+ (area 3 4 5) 1))",
    ));
}

#[test]
fn cli_writes_add_function_parameter_for_scheme_start() {
    let dir = fresh_temp_dir("add-function-parameter");
    let scheme_file = dir.join("render.scm");
    fs::write(
        &scheme_file,
        "(define (area width height) (* width height))\n(define rendered (area 10 20))\n",
    )
    .expect("write scheme fixture");

    let mut cmd = paredit();
    cmd.arg("add-function-parameter")
        .arg("--file")
        .arg(&scheme_file)
        .arg("--definition-path")
        .arg("0")
        .arg("--name")
        .arg("scale")
        .arg("--argument")
        .arg("2")
        .arg("--call-path")
        .arg("1.2")
        .arg("--insert")
        .arg("start")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"function_name\": \"area\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(scheme_file).expect("read parameterized scheme"),
        "(define (area scale width height) (* width height))\n(define rendered (area 2 10 20))\n"
    );
}

#[test]
fn cli_rejects_add_function_parameter_mismatched_call() {
    let mut cmd = paredit();
    cmd.args([
        "add-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--argument",
        "5",
        "--call-path",
        "1.3",
    ])
    .write_stdin(
        "(defun area (width height) (* width height))\n(defun render () (perimeter 10 20))",
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "does not match selected definition",
    ));
}

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

#[test]
fn cli_rejects_function_parameter_all_calls_with_explicit_call_path() {
    let mut cmd = paredit();
    cmd.args([
        "remove-function-parameter",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--all-calls",
        "--call-path",
        "1.3",
    ])
    .write_stdin(
        "(defun area (width height margin) (* width height))\n(defun render () (area 10 20 5))",
    )
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "either --all-calls or repeated --call-path",
    ));
}
