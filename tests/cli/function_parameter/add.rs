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
fn cli_plans_add_function_parameter_for_common_lisp_defmethod() {
    let mut cmd = paredit();
    cmd.args([
        "add-function-parameter",
        "--dialect",
        "common-lisp",
        "--definition-path",
        "0",
        "--name",
        "style",
        "--argument",
        ":fancy",
        "--call-path",
        "1",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defmethod render :around ((node widget) stream) (draw node stream))\n(render thing out)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"function_name\": \"render\""))
    .stdout(predicate::str::contains("\"parameter_name\": \"style\""))
    .stdout(predicate::str::contains(
        "(defmethod render :around ((node widget) stream style)",
    ))
    .stdout(predicate::str::contains("(render thing out :fancy)"));
}

#[test]
fn cli_plans_add_function_parameter_for_common_lisp_key_parameter() {
    let mut cmd = paredit();
    cmd.args([
        "add-function-parameter",
        "--dialect",
        "common-lisp",
        "--definition-path",
        "0",
        "--name",
        "margin",
        "--argument",
        "8",
        "--call-path",
        "1",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render (node &key color) (list node color margin))\n(render item :color :red)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"parameter_name\": \"margin\""))
    .stdout(predicate::str::contains(
        "(defun render (node &key color margin)",
    ))
    .stdout(predicate::str::contains(
        "(render item :color :red :margin 8)",
    ));
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
fn cli_plans_add_function_parameter_all_calls_without_common_lisp_labels_shadowed_calls() {
    let mut cmd = paredit();
    cmd.args([
        "add-function-parameter",
        "--dialect",
        "common-lisp",
        "--definition-path",
        "0",
        "--name",
        "b",
        "--argument",
        "0",
        "--all-calls",
        "--output",
        "json",
    ])
    .write_stdin(
        "\
(defun f (a) a)
(defun caller ()
  (labels ((f (x) (f x))
           (g (y) (f y)))
    (f 1)
    (cl:print (f 2)))
  (f 3))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"all_calls\": true"))
    .stdout(predicate::str::contains(
        "\"call_paths\": [\n    \"1.4\"\n  ]",
    ))
    .stdout(predicate::str::contains("(defun f (a b) a)"))
    .stdout(predicate::str::contains("(f x))"))
    .stdout(predicate::str::contains("(f 1)"))
    .stdout(predicate::str::contains("(f 3 0)"));
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
