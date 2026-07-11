use super::*;

#[test]
fn cli_writes_move_function_parameter_for_scheme() {
    let dir = fresh_temp_dir("move-function-parameter");
    let scheme_file = dir.join("render.scm");
    fs::write(
        &scheme_file,
        "(define (area width height scale) (* scale width height))\n(define rendered (area 10 20 2))\n",
    )
    .expect("write scheme fixture");

    let mut cmd = move_command();
    cmd.arg("--file")
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

    let output = move_command()
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
        .arg("--output")
        .arg("json")
        .output()
        .expect("run move-function-parameter");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_move_function_parameter_report(&output.stdout).expect("parse move report");
    assert!(report.all_calls);
    assert!(report.changed);
    assert!(report.written);
    assert_eq!(report.function_name, "area");
    assert_eq!(report.parameter_name, "scale");
    assert_eq!(report.from_index, 0);
    assert_eq!(report.to_index, 2);
    assert_eq!(report.moved_arguments, vec!["2".to_owned(), "3".to_owned()]);

    let rewritten =
        fs::read_to_string(common_lisp_file).expect("read parameter-reordered common lisp");
    assert_eq!(rewritten, report.rewritten);
    assert_eq!(
        rewritten,
        "(defun area (width height scale) (* scale width height))\n(defun a () (area 10 20 2))\n(defun b () (list (area 4 5 3)))\n"
    );
}
