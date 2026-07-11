use super::*;

#[test]
fn cli_writes_reorder_function_parameters_with_all_calls() {
    let dir = fresh_temp_dir("reorder-function-parameters-all-calls");
    let common_lisp_file = dir.join("render.lisp");
    fs::write(
        &common_lisp_file,
        "(defun area (scale width height) (* scale width height))\n(defun a () (area 2 10 20))\n(defun b () (list (area 3 4 5)))\n",
    )
    .expect("write common lisp fixture");

    let output = reorder_command()
        .arg("--file")
        .arg(&common_lisp_file)
        .arg("--definition-path")
        .arg("0")
        .args([
            "--parameter",
            "height",
            "--parameter",
            "scale",
            "--parameter",
            "width",
        ])
        .arg("--all-calls")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run reorder-function-parameters");

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report =
        parse_reorder_function_parameters_report(&output.stdout).expect("parse reorder report");
    assert!(report.all_calls);
    assert!(report.changed);
    assert!(report.written);
    assert_eq!(report.function_name, "area");
    assert_eq!(
        report.old_parameter_order,
        vec!["scale".to_owned(), "width".to_owned(), "height".to_owned()]
    );
    assert_eq!(
        report.new_parameter_order,
        vec!["height".to_owned(), "scale".to_owned(), "width".to_owned()]
    );
    assert_eq!(report.reordered_arguments.len(), 2);

    let rewritten =
        fs::read_to_string(common_lisp_file).expect("read parameter-reordered common lisp");
    assert_eq!(rewritten, report.rewritten);
    assert_eq!(
        rewritten,
        "(defun area (height scale width) (* scale width height))\n(defun a () (area 20 2 10))\n(defun b () (list (area 5 3 4)))\n"
    );
}
